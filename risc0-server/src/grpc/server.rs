use std::{io::Read, str::FromStr};

use flate2::read::ZlibDecoder;
use hex::FromHex;
use serde_json::Value;
use tonic::{Request, Response, Status};

use crate::{db, handlers::proof::get_receipt, model::models::ProofType};

use rust_grpc::grpc::vm_runtime::{
    vm_runtime_server::VmRuntime, CreateRequest, CreateResponse, ExecuteRequest, ExecuteResponse,
};

#[derive(Debug)]
pub struct Risc0Server {}

#[tonic::async_trait]
impl VmRuntime for Risc0Server {
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        println!("risc0 instance create...");
        let request = request.into_inner();

        let project_id = request.project_id;
        let project = project_id.to_string();
        let content = request.content;
        let compressed_data = Vec::from_hex(content).unwrap();
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut content = Vec::new();
        decoder.read_to_end(&mut content)?;

        let exp_param = match request.exp_params.get(0) {
            Some(p) => p,
            None => return Err(Status::invalid_argument("need exp_param")),
        };

        // exp_param = {"image_id":"RANGE_ID", "elf":"RANGE_ELF"}
        if exp_param == "" {
            return Err(Status::invalid_argument("need exp_param"));
        }

        let v: Value = serde_json::from_str(&exp_param).unwrap();
        let image_id = v["image_id"].as_str().unwrap().to_string();
        let elf = v["elf"].as_str().unwrap().to_string();
        let content = String::from_utf8(content).unwrap();

        let mut elf_str = String::new();
        let mut id_str = String::new();

        for line in content.lines() {
            if line.contains(&elf) {
                let vec: Vec<&str> = line.split("=").collect();
                elf_str = vec[1].trim()[2..vec[1].trim().len() - 2].to_string();
            }
            if line.contains(&image_id) {
                let vec: Vec<&str> = line.split("=").collect();
                id_str = vec[1].trim()[1..vec[1].trim().len() - 2].to_string();
            }
        }

        let connection = &mut db::pgdb::establish_connection();
        let _ = db::pgdb::create_vm(connection, &project, &elf_str, &id_str);

        Ok(Response::new(CreateResponse {}))
    }

    async fn execute_operator(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        println!("risc0 instance execute...");
        let request = request.into_inner();

        let project_id = request.project_id;
        let task_id = request.task_id;
        let client_id = request.client_id;
        let sequencer_signature = request.sequencer_signature;
        let datas = request.datas;

        if datas.len() == 0 {
            return Err(Status::invalid_argument("need datas"));
        }

        let connection = &mut db::pgdb::establish_connection();
        let image_id = match db::pgdb::get_vm_by_project(connection, &project_id.to_string()) {
            Ok(v) => v.image_id,
            Err(_) => {
                return Err(Status::not_found(format!(
                    "{} not found",
                    project_id.to_string()
                )))
            }
        };

        // TODO move to guest method
        // param = {"private_input":"14", "public_input":"3,34", "receipt_type":"Snark"}
        // let input_datas = json!(datas).to_string();
        let v: Value = serde_json::from_str(&datas[0]).unwrap();
        let mut receipt_type = ProofType::from_str("Snark").unwrap();
        // TODO check v
        if v.get("receipt_type").is_some() {
            receipt_type = v["receipt_type"]
                .as_str()
                .unwrap()
                .to_string()
                .parse()
                .unwrap();
        }
        // let receipt_type: Result<ProofType, _> =
        //     v["receipt_type"].as_str().unwrap().to_string().parse();
        // let receipt_type = receipt_type.unwrap();

        let receipt = match get_receipt(
            project_id,
            task_id,
            client_id,
            sequencer_signature,
            image_id,
            datas,
            receipt_type,
        )
        .await
        {
            Ok(r) => r,
            Err(e) => return Err(Status::cancelled(e)),
        };

        Ok(Response::new(ExecuteResponse {
            result: receipt.as_bytes().to_vec(),
        }))
    }
}
