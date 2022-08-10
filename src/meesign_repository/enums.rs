use diesel_derive_enum::DbEnum;



#[derive(Copy, Clone, PartialEq, Eq, Debug, DbEnum)]
#[DieselType = "Protocoltype"]
pub enum ProtocolType {
    GG18
}

#[derive(Debug, DbEnum)]
#[DieselType = "TaskType"]
pub enum Tasktype {
    Group, Sign
}

#[derive(Debug, DbEnum)]
#[DieselType = "Taskresulttype"]
pub enum TaskResultType {
    GroupEstablished, Signed
}

#[derive(Debug, DbEnum)]
#[DieselType = "Taskstate"]
pub enum TaskState {
    Created, Running, Finished, Failed
}