mod error;

use crate::error::Error;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Output;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn package_parser(github_url: &str, rev: &str, subdir: &str) -> Result<String, Error> {
    let github_url = github_url
        .strip_prefix("https://github.com")
        .map(|g| g.strip_suffix(".git").unwrap_or(g))
        .unwrap_or_default();

    let url = format!(
        "{}/{}/{}/{}/{}",
        std::env::var("GITHUB_RAW_BASE").map_err(|_| Error::NotFound {
            msg: "env GITHUB_RAW_BASE not found".to_string()
        })?,
        github_url,
        rev,
        subdir,
        "Move.toml"
    );

    let res = reqwest::get(url)
        .await
        .map_err(|e| Error::AnyError(anyhow::Error::new(e)))?;

    let package: PackageWrapper = toml::from_str(&res.text().await.unwrap_or_default())
        .map_err(|e| Error::AnyError(anyhow::Error::new(e)))?;

    Ok(package.package.name)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

impl Package {
    pub fn new(name: String, version: String) -> Self {
        Package { name, version }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageWrapper {
    package: Package,
}

/// placed in Move.toml
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub git: String,
    pub rev: String,
    pub subdir: String,
}

impl Dependency {
    pub fn new(git: &str, rev: &str, subdir: &str) -> Self {
        Dependency {
            git: git.to_string(),
            rev: rev.to_string(),
            subdir: subdir.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// address::module_name::function_name
    pub full_path: String,
    /// Generic params i.g) 0x1::aptos_account::AptosCoin
    pub type_arguments: Vec<String>,
    pub arguments: Vec<String>,
}

impl Function {
    pub fn new(full_path: &str, type_arguments: Vec<String>, arguments: Vec<String>) -> Self {
        Function {
            full_path: full_path.to_string(),
            type_arguments,
            arguments,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileResult {
    pub dir: PathBuf,
    pub output: Output,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MoveScript {
    pub dir: PathBuf,
    pub dependencies: Vec<Dependency>,
    pub functions: Vec<Function>,
    pub generated_main_function: String,
}

impl MoveScript {
    pub fn new() -> Self {
        Self::default()
    }
    /// set temporal directory name
    pub fn init(mut self) -> Self {
        self.dir = PathBuf::new();
        self.dir
            .push(std::env::var("MOVE_SCRIPT_DEFAULT_PATH").unwrap_or_default());
        self.dir.push(Uuid::new_v4().to_string());
        self
    }

    pub fn add_dependency(mut self, dep: Dependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    pub fn add_dependencies(mut self, deps: Vec<Dependency>) -> Self {
        deps.into_iter().for_each(|d| self.dependencies.push(d));
        self
    }

    pub fn add_function(mut self, function: Function) -> Self {
        self.functions.push(function);
        self
    }

    pub fn add_functions(mut self, functions: Vec<Function>) -> Self {
        functions.into_iter().for_each(|f| self.functions.push(f));
        self
    }

    async fn create_base_dir(&self) -> Result<(), Error> {
        if self.dir == PathBuf::new() {
            return Err(Error::NotFound {
                msg: String::from("directory name not set"),
            });
        }
        tokio::fs::create_dir_all(self.dir.as_path())
            .await
            .map_err(|e| Error::Generate { msg: e.to_string() })?;

        tokio::fs::create_dir(self.dir.as_path().join("sources"))
            .await
            .map_err(|e| Error::Generate { msg: e.to_string() })?;

        Ok(())
    }

    async fn generate_toml(&mut self) -> Result<(), Error> {
        let package = PackageWrapper {
            package: Package::new("block-stack".to_string(), "1.0.0".to_string()),
        };

        let str_package =
            toml::to_string(&package).map_err(|e| Error::Generate { msg: e.to_string() })?;

        let str_dependencies_task = self
            .dependencies
            .iter()
            .map(|dep| async {
                let deps_name = package_parser(&dep.git, &dep.rev, &dep.subdir).await;
                match deps_name {
                    Ok(deps_name) => format_args!(
                        "{} = {{ git = '{}', rev = '{}', subdir = '{}' }}\n",
                        deps_name, dep.git, dep.rev, dep.subdir
                    )
                    .to_string(),
                    Err(_) => String::new(),
                }
            })
            .collect::<Vec<_>>();

        let str_dependencies: Vec<String> = join_all(str_dependencies_task).await;

        let mut file = tokio::fs::File::create(self.dir.join("Move.toml").as_path()).await?;
        file.write_all(str_package.as_bytes()).await?;
        file.write_all(b"\n[dependencies]\n").await?;

        for dep in str_dependencies {
            file.write_all(dep.as_bytes()).await?;
        }

        Ok(())
    }

    async fn generate_main_function(&mut self) -> Result<(), Error> {
        let mut content = String::new();

        for function in &self.functions {
            let mut type_ars = "<".to_string();
            for ta in &function.type_arguments {
                type_ars.push_str(ta);
                type_ars.push(',');
            }
            type_ars.push('>');

            content.push_str(&function.full_path);
            content.push_str(&type_ars);
            content.push_str("(&user, ");

            for arg in &function.arguments {
                if arg.starts_with("0x") {
                    content.push_str(&format!("@{},", arg));
                } else {
                    content.push_str(&format!("{},", arg));
                }
            }
            content.push_str(");\n\t\t");
        }

        let main_function = format!("fun main(user: signer) {{\n\t\t{}\n\t}}", content);

        self.generated_main_function = main_function;
        Ok(())
    }

    async fn wrap_to_script(&self) -> Result<(), Error> {
        let script = format!(
            "script {{\n\t{}\n}}
        ",
            self.generated_main_function
        );

        let mut file = tokio::fs::File::create(self.dir.join("sources/script.move")).await?;
        file.write_all(script.as_bytes()).await?;

        Ok(())
    }

    pub async fn generate_script(&mut self) -> Result<CompileResult, Error> {
        self.create_base_dir().await?;
        self.generate_toml().await?;
        self.generate_main_function().await?;
        self.wrap_to_script().await?;
        self.compile()
    }

    pub async fn destroy_self(self) -> Result<(), Error> {
        Ok(fs::remove_dir_all(self.dir).await?)
    }

    pub fn compile(&self) -> Result<CompileResult, Error> {
        let dir = self.dir.as_path().to_str().unwrap_or_default().to_string();

        let output = std::process::Command::new("aptos")
            .arg("move")
            .arg("compile")
            .arg("--package-dir")
            .arg(dir)
            .output()
            .map_err(|_| Error::Generate {
                msg: "script compile failed".to_string(),
            })?;

        Ok(CompileResult {
            dir: self.dir.clone(),
            output,
        })
    }
}

#[cfg(test)]
mod script {
    use crate::{Dependency, Function, MoveScript};
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_script_generator() {
        dotenv().ok();
        let mut script = MoveScript::new()
            .init()
            .add_dependency(Dependency::new(
                "https://github.com/aptos-labs/aptos-core",
                "main",
                "aptos-move/framework/aptos-framework",
            ))
            .add_function(Function::new(
                "0x1::coin::transfer",
                vec!["0x1::aptos_coin::AptosCoin".to_string()],
                vec![
                    "0x5a5e4bf66077215d385c2178e9a4ce2321c5f8cdc79ca849064dece5b36ce308"
                        .to_string(),
                    "100000".to_string(),
                ],
            ))
            .add_function(Function::new(
                "0x1::coin::transfer",
                vec!["0x1::aptos_coin::AptosCoin".to_string()],
                vec![
                    "0x5a5e4bf66077215d385c2178e9a4ce2321c5f8cdc79ca849064dece5b36ce308"
                        .to_string(),
                    "100000".to_string(),
                ],
            ));

        script.generate_script().await.unwrap();
        script.destroy_self().await.unwrap();
    }
}
