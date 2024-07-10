use std::{
    env,
    fs::{self},
    sync::RwLock,
};

use diesel::connection::SimpleConnection;
use lazy_static::lazy_static;
use rust_grpc::grpc::vm_runtime::{
    vm_runtime_client::VmRuntimeClient, CreateRequest, ExecuteRequest,
};
use serde_json::Value;
use tonic::{transport::Channel, Request};

use crate::{db, tests::mock::Server};

mod mock;

lazy_static! {
    static ref SERVER: RwLock<Server> = RwLock::new(Server::new());
}

async fn init_db() {
    env::set_var(
        "DATABASE_URL",
        "postgres://test_user:test_passwd@127.0.0.1:15432/test?sslmode=disable",
    );
    let init_sql = r#"
    DROP TABLE IF EXISTS vms;
    CREATE TABLE IF NOT EXISTS vms (
		id SERIAL PRIMARY KEY,
		project_name VARCHAR NOT NULL,
		elf TEXT NOT NULL,
		image_id VARCHAR NOT NULL
	  );
	DROP TABLE IF EXISTS proofs;  
	CREATE TABLE IF NOT EXISTS proofs (
        id SERIAL PRIMARY KEY,
        project_id VARCHAR NOT NULL,
        task_id VARCHAR NOT NULL,
        client_id VARCHAR NOT NULL,
        sequencer_sign VARCHAR NOT NULL,
        image_id VARCHAR NOT NULL,
        datas_input VARCHAR NOT NULL,
        receipt_type VARCHAR NOT NULL,
        receipt TEXT,
        status VARCHAR NOT NULL,
        create_at TIMESTAMP NOT NULL DEFAULT now()
	)"#;
    let connection = &mut db::pgdb::establish_connection();
    connection.batch_execute(init_sql).unwrap();
}

async fn init_real_server() {
    let _ = init_db().await;
    SERVER.write().unwrap().init_server().await;
}

#[tokio::test]
async fn test_create_and_execute_e2e() {
    init_real_server().await;

    // create client
    let channel = Channel::from_static("http://127.0.0.1:14001")
        .connect()
        .await
        .unwrap();
    let mut client = VmRuntimeClient::new(channel);

    // create vm instance
    let file_content = fs::read_to_string("./src/tests/10000.json").unwrap();
    let v: Value = serde_json::from_str(&file_content).unwrap();
    let content = v["code"].as_str().unwrap().to_string();
    let exp_params = v["codeExpParams"].as_array().unwrap();
    let exp_params = exp_params
        .iter()
        .map(|param| param.as_str().unwrap().to_string())
        .collect();
    let project_id: u64 = 10000;

    let req = Request::new(CreateRequest {
        project_id,
        content,
        exp_params,
    });

    let response = client.create(req).await;
    match response {
        Ok(_) => assert!(true),
        Err(err) => {
            println!("failed to create vm instance, {:?}", err);
            assert!(false)
        }
    }

    // generate a proof
    let req = Request::new(ExecuteRequest {
        project_id,
        task_id: 0u64,
        client_id: "test_client_id".to_string(),
        sequencer_signature: "test_sequencer_sign".to_string(),
        datas: vec![
            "{\"private_input\":\"14\", \"public_input\":\"3,34\", \"receipt_type\":\"Stark\"}"
                .to_string(),
        ],
    });
    let response = client.execute_operator(req).await;
    match response {
        Ok(_) => assert!(true),
        Err(err) => {
            println!("failed to executor vm instance, {:?}", err);
            assert!(false)
        }
    }
}

#[tokio::test]
async fn test_create_failed_e2e() {
    init_real_server().await;

    // create client
    let channel = Channel::from_static("http://127.0.0.1:14001")
        .connect()
        .await
        .unwrap();
    let mut client = VmRuntimeClient::new(channel);

    // create vm instance
    let file_content = fs::read_to_string("./src/tests/10000.json").unwrap();
    let v: Value = serde_json::from_str(&file_content).unwrap();
    let content = v["code"].as_str().unwrap().to_string();
    let exp_params = vec!["".to_string()];
    let project_id: u64 = 10000;

    let req = Request::new(CreateRequest {
        project_id,
        content,
        exp_params,
    });

    let response = client.create(req).await;
    match response {
        Ok(_) => (),
        Err(err) => {
            assert_eq!("need exp_param", err.message());
        }
    }
}

#[tokio::test]
async fn test_executor_failed_e2e() {
    init_real_server().await;

    // create client
    let channel = Channel::from_static("http://127.0.0.1:14001")
        .connect()
        .await
        .unwrap();
    let mut client = VmRuntimeClient::new(channel);

    // create vm instance
    let file_content = fs::read_to_string("./src/tests/10000.json").unwrap();
    let v: Value = serde_json::from_str(&file_content).unwrap();
    let content = v["code"].as_str().unwrap().to_string();
    let exp_params = v["codeExpParams"].as_array().unwrap();
    let exp_params = exp_params
        .iter()
        .map(|param| param.as_str().unwrap().to_string())
        .collect();
    let project_id: u64 = 10000;

    let req = Request::new(CreateRequest {
        project_id,
        content,
        exp_params,
    });

    let response = client.create(req).await;
    match response {
        Ok(_) => assert!(true),
        Err(err) => {
            println!("failed to create vm instance, {:?}", err);
            assert!(false)
        }
    }

    // datas is nil
    let req = Request::new(ExecuteRequest {
        project_id,
        task_id: 0u64,
        client_id: "test_client_id".to_string(),
        sequencer_signature: "test_sequencer_sign".to_string(),
        datas: vec![],
    });
    let response = client.execute_operator(req).await;
    match response {
        Ok(_) => (),
        Err(err) => {
            assert_eq!("need datas", err.message())
        }
    }

    // project not found
    let req = Request::new(ExecuteRequest {
        project_id: 99999,
        task_id: 0u64,
        client_id: "test_client_id".to_string(),
        sequencer_signature: "test_sequencer_sign".to_string(),
        datas: vec![
            "{\"private_input\":\"14\", \"public_input\":\"3,34\", \"receipt_type\":\"Stark\"}"
                .to_string(),
        ],
    });
    let response = client.execute_operator(req).await;
    match response {
        Ok(_) => (),
        Err(err) => {
            assert_eq!("99999 not found", err.message())
        }
    }
}
