use crate::list::List;

pub enum DataType{
    STRING(String),
    LIST(List)
}
pub struct RedisObject{
    pub data:DataType,

}