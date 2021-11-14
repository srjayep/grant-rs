use crate::util;
use anyhow::{Context, Error, Result, *};
use log::{debug, info};
use std::path::PathBuf;
use yaml_rust::yaml::Hash;
use yaml_rust::Yaml;

pub fn apply(file: &PathBuf, dryrun: bool, conn: Option<String>) {
    info!(
        "Try to apply definition from {:?}, dryrun={}, conn={:?}",
        file, dryrun, conn
    );

    let config = util::read_config(file).unwrap();
    // debug!("Roles: {:#?}", &config["roles"][0]);

    for user in config["users"].as_vec().expect("must have users").iter() {
        let _current_user = user.as_hash().unwrap();

        let _name = _current_user
            .get(&Yaml::from_str("name"))
            .expect("users[*].name is required")
            .as_str()
            .unwrap();

        let _current_roles: Vec<_> = _current_user
            .get(&Yaml::from_str("roles"))
            .expect("users[*].roles is required")
            .as_vec()
            .unwrap()
            .iter()
            .map(|role_name| {
                role_name
                    .as_hash()
                    .unwrap()
                    .get(&Yaml::from_str("name"))
                    .expect("roles[*].name is required")
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .map(|role_name| lookup_role(&config, role_name).unwrap())
            .collect();

        for role in _current_roles.iter() {
            let sql = format!(
                "GRANT {} TO {};",
                generate_sql_by_role(&role).unwrap(),
                _name
            );
            debug!("SQL = {}", sql);
        }
    }
}

fn lookup_role<'a>(config: &'a Yaml, name: String) -> Result<&'a Hash> {
    for role in config["roles"].as_vec().unwrap().iter() {
        let role_name = role
            .as_hash()
            .unwrap()
            .get(&Yaml::from_str("name"))
            .unwrap()
            .as_str()
            .unwrap();
        if role_name == name {
            return Ok(role.as_hash().unwrap());
        }
    }

    Err(anyhow!("role {} not found", name))
}

fn generate_sql_by_role(role: &Hash) -> Result<String> {
    // debug!("Role: {:#?}", role);

    let level = role
        .get(&Yaml::from_str("level"))
        .expect("level is required")
        .as_str()
        .unwrap();

    let grants: Vec<_> = role
        .get(&Yaml::from_str("grants"))
        .expect("grants is required")
        .as_vec()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();

    let databases: Vec<_> = role
        .get(&Yaml::from_str("databases"))
        .expect("databases is required")
        .as_vec()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();

    let default_schemas = Yaml::Array(Vec::new());
    let schemas: Vec<_> = role
        .get(&Yaml::from_str("schemas"))
        .unwrap_or(&default_schemas)
        .as_vec()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();

    let default_tables = Yaml::Array(Vec::new());
    let tables: Vec<_> = role
        .get(&Yaml::from_str("tables"))
        .unwrap_or(&default_tables)
        .as_vec()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();

    match level {
        "DATABASE" => Ok(grant_database(&grants, &databases).unwrap()),
        "SCHEMA" => Ok(grant_schema(&grants, &schemas).unwrap()),
        "TABLE" => Ok(grant_table(&grants, &schemas, &tables).unwrap()),
        _ => Err(anyhow!("Invalid `level`")),
    }
}

// GRANT { { CREATE | TEMPORARY | TEMP } [,...] | ALL [ PRIVILEGES ] }
// ON DATABASE db_name [, ...]
// TO { username [ WITH GRANT OPTION ] | GROUP group_name | PUBLIC } [, ...]
fn grant_database(grants: &Vec<&str>, databases: &Vec<&str>) -> Result<String> {
    // TODO: print Vec<&str>
    let grants_str = if grants.iter().any(|&i| i == "ALL" || i == "*") {
        "ALL PRIVILEGES".to_string()
    } else {
        grants.join(", ")
    };
    let databases_str = databases.join(", ");
    let sql = format!("{} ON DATABASE {}", grants_str, databases_str);

    Ok(sql)
}

// GRANT { { CREATE | USAGE } [,...] | ALL [ PRIVILEGES ] }
// ON SCHEMA schema_name [, ...]
// TO { username [ WITH GRANT OPTION ] | GROUP group_name | PUBLIC } [, ...]
fn grant_schema(grants: &Vec<&str>, schemas: &Vec<&str>) -> Result<String> {
    let grants_str = if grants.iter().any(|&i| i == "ALL" || i == "*") {
        "ALL PRIVILEGES".to_string()
    } else {
        grants.join(", ")
    };
    let schemas_str = schemas.join(", ");

    let sql = format!("{} ON SCHEMA {:?}", grants_str, schemas_str);

    Ok(sql)
}

// GRANT { { SELECT | INSERT | UPDATE | DELETE | DROP | REFERENCES } [,...] | ALL [ PRIVILEGES ] }
// ON { [ TABLE ] table_name [, ...] | ALL TABLES IN SCHEMA schema_name [, ...] }
// TO { username [ WITH GRANT OPTION ] | GROUP group_name | PUBLIC } [, ...]
fn grant_table(grants: &Vec<&str>, schemas: &Vec<&str>, tables: &Vec<&str>) -> Result<String> {
    let grants_str = if grants.iter().any(|&i| i == "ALL" || i == "*") {
        "ALL PRIVILEGES".to_string()
    } else {
        grants.join(", ")
    };

    let schemas_str = schemas.join(", ");
    let tables_str = if tables.iter().any(|&i| i == "ALL" || i == "*") {
        format!("ALL TABLES IN SCHEMA {}", schemas_str)
    } else {
        format!("TABLE {} IN SCHEMA {}", tables.join(", "), schemas_str)
    };

    let sql = format!("{} ON {}", grants_str, tables_str);

    Ok(sql)
}
