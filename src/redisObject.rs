use std::collections::HashMap;

use crate::list::List;

pub enum DataType{
    STRING(String),
    LIST(List),
    STREAM(HashMap<String,HashMap<String,String>>)
}
pub struct RedisObject{
    pub data:DataType,

}