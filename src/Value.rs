
#![allow(dead_code)]

use crate::AST::expr_node::DataType;
#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Value {
    Null,
    Integer(i128),
    Real(f64),
    Str(String),
    Object(String),
    Char(char),
    Boolean(bool),
    List(Vec<Value>)
}
#[derive(Debug,Clone,PartialEq)]
pub struct TypedValue {
    pub val: Value,
    pub val_type: DataType,
    pub is_ptr: bool
}

impl TypedValue {
    pub fn new(val: Value, is_ptr: bool) -> Self {
        TypedValue { val:val.clone(), val_type: val.to_datatype(), is_ptr }
    }
    pub fn cast(&mut self, new_datatype: DataType) -> Result<(), String> {
        if self.val_type == new_datatype {return Ok(());}
        self.val_type = new_datatype;
        self.val = match self.val_type {
            DataType::I8 => Value::Char(
                unsafe {
                    char::from_u32_unchecked(self.val.clone().to_literal()? as u32)
                }
            ),
            DataType::I16 => Value::Integer(
                match self.val.clone() {
                    Value::Integer(n) => n as i128 & i16::MAX as i128,
                    Value::Real(n) => n as i128 & i16::MAX as i128,
                    o => return Err(format!("Cannot cast {:#?} to i16",o))
                }
            ),
            DataType::I32 => Value::Integer(
                match self.val.clone() {
                    Value::Integer(n) => n as i128 & i32::MAX as i128,
                    Value::Real(n) => n as i128 & i32::MAX as i128,
                    o => return Err(format!("Cannot cast {:#?} to i32",o))
                }
            ),
            DataType::I64 => Value::Integer(
                match self.val.clone() {
                    Value::Integer(n) => n as i128 & i64::MAX as i128,
                    Value::Real(n) => n as i128 & i64::MAX as i128,
                    o => return Err(format!("Cannot cast {:#?} to i64",o))
                }
            ),
            DataType::U8 => Value::Integer(
                match self.val.clone() {
                    Value::Integer(n) => n as i128 & u8::MAX as i128,
                    Value::Real(n) => n as i128 & u8::MAX as i128,
                    o => return Err(format!("Cannot cast {:#?} to u8",o))
                }
            ),
            DataType::U16 => Value::Integer(
                match self.val.clone() {
                    Value::Integer(n) => n as i128 & u16::MAX as i128,
                    Value::Real(n) => n as i128 & u16::MAX as i128,
                    o => return Err(format!("Cannot cast {:#?} to u16",o))
                }
            ),
            DataType::U32 => Value::Integer(
                
                match self.val.clone() {
                    Value::Integer(n) => n as i128 & u32::MAX as i128,
                    Value::Real(n) => n as i128 & u32::MAX as i128,
                    o => return Err(format!("Cannot cast {:#?} to u32",o))
                }
            ),
            DataType::U64 => Value::Integer(
                
                match self.val.clone() {
                    Value::Integer(n) => n as i128 & u64::MAX as i128,
                    Value::Real(n) => n as i128 & u64::MAX as i128,
                    o => return Err(format!("Cannot cast {:#?} to u64",o))
                }
            ),
            DataType::F32 => Value::Real(
                match self.val.clone() {
                    Value::Integer(n) => n as f64 % f32::MAX as f64,
                    Value::Real(n) => n % f32::MAX as f64,
                    o => return Err(format!("Cannot cast {:#?} to f32",o))
                }
            ),
            DataType::F64 => Value::Real(
                match self.val.clone() {
                    Value::Integer(n) => n as f64 % f64::MAX,
                    Value::Real(n) => n,
                    o => return Err(format!("Cannot cast {:#?} to f64",o))
                }
            ),
            DataType::Void => unimplemented!("void cannot be cast to anything!"),
            DataType::Unknown => todo!(),
        };
        Ok(())
    }
}

impl std::ops::Add for Value {
    type Output = Result<Value,String>;
    fn add(self, rhs: Self) -> Self::Output {

        if let (&Value::Integer(lhs_n), &Value::Integer(rhs_n)) = (&self,&rhs) {
            Ok(Value::Integer(lhs_n+rhs_n))
        } else if let (Value::Str(s1), Value::Str(s2)) = (&self,&rhs) {
            Ok(Value::Str(format!("{}{}", s1,s2)))
        }
        else {
            Err("Not supported yet".to_string())
        }

        /*
        if matches!(self, Value::Integer(_)) && matches!(rhs, Value::Integer(_)) {
            Value::Integer(self.to_literal()+rhs.to_literal())
        } else if matches!(self, Value::Str(_)) && matches!(rhs, Value::Str(_)) {
            Value::Str(format!("{}{}", self.to_string(),rhs.to_string()))
        } else if matches!(self, Value::Real(_)) || matches!(rhs, Value::Real(_)) {
            Value::Real(self.to_float()+rhs.to_float())
        }
        else {
            Value::Null
        }*/
    }
}

impl std::ops::Neg for Value {
    type Output = Result<Value,String>;
    fn neg(self) -> Self::Output {
        match self {
            Value::Real(f)   => Ok(Value::Real(-f)),
            Value::Integer(n) => Ok(Value::Integer(-n)),
            _ => Err("[NEG]: Negative number only apply for integer, float or double".to_string())
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Result<Value,String>;
    fn sub(self, rhs: Self) -> Self::Output {
        if let (&Value::Integer(lhs_n), &Value::Integer(rhs_n)) = (&self,&rhs) {
            Ok(Value::Integer(lhs_n-rhs_n))
        }
        else {
            Err("[SUB] Both value MUST Be integer or float.".to_string())
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Result<Value,String>;
    fn mul(self, rhs: Self) -> Self::Output {
        if let (&Value::Integer(lhs_n), &Value::Integer(rhs_n)) = (&self,&rhs) {
            Ok(Value::Integer(lhs_n*rhs_n))
        }
        else {
            Err("[MUL] Both value MUST Be integer or float.".to_string())
        }
    }
}

impl std::ops::Not for Value {
    type Output = Result<Value,String>;
    fn not(self) -> Self::Output {
        if let &Value::Integer(n) = &self {
            Ok(Value::Integer(!n))
        } else if let &Value::Boolean(n) = &self {
            Ok(Value::Boolean(!n))
        } else {
            Err(format!("[NOT] It can only apply to integer, not {:?}", self))
        }
    }
}

impl std::ops::Shl for Value {
    type Output = Result<Value,String>;
    fn shl(self, rhs: Self) -> Self::Output {
        let (&Value::Integer(lhs_n), &Value::Integer(rhs_n)) = (&self,&rhs) else {
            return Err("[SHL] Cannot shl on non-integer value".to_string());
        };

        if lhs_n < 0 || rhs_n<0 { 
            Err("[SHL] value must be natural number".to_string())
        } else {
            Ok(Value::Integer(lhs_n<<rhs_n))
        }
    }
}
impl std::ops::BitOr for Value {
    type Output = Result<Value,String>;
    fn bitor(self, rhs: Self) -> Self::Output {
        let (&Value::Integer(lhs_n), &Value::Integer(rhs_n)) = (&self,&rhs) else {
            return Err("[BITOR] Cannot 'or' on non-integer value".to_string());
        };

        if lhs_n < 0 || rhs_n<0 { 
            Err("[BITOR] value must be natural number".to_string())
        } else {
            Ok(Value::Integer(lhs_n|rhs_n))
        }
    }
}

impl std::ops::BitAnd for Value {
    type Output = Result<Value,String>;
    fn bitand(self, rhs: Self) -> Self::Output {
        let (&Value::Integer(lhs_n), &Value::Integer(rhs_n)) = (&self,&rhs) else {
            return Err("[BITAND] Cannot 'and' on non-integer value".to_string());
        };

        if lhs_n < 0 || rhs_n<0 { 
            Err("[BITAND] value must be natural number".to_string())
        } else {
            Ok(Value::Integer(lhs_n&rhs_n))
        }
    }
}
impl std::ops::BitXor for Value {
    type Output = Result<Value,String>;
    fn bitxor(self, rhs: Self) -> Self::Output {
        let (&Value::Integer(lhs_n), &Value::Integer(rhs_n)) = (&self,&rhs) else {
            return Err("[BITXOR] Cannot 'xor' on non-integer value".to_string());
        };

        if lhs_n < 0 || rhs_n<0 { 
            Err("[BITXOR] value must be natural number".to_string())
        } else {
            Ok(Value::Integer(lhs_n^rhs_n))
        }
    }
}

impl std::ops::Shr for Value {
    type Output = Result<Value,String>;
    fn shr(self, rhs: Self) -> Self::Output {
        let (&Value::Integer(lhs_n), &Value::Integer(rhs_n)) = (&self,&rhs) else {
            return Err("[SHR] Cannot shl on non-integer value".to_string());
        };

        if lhs_n < 0 || rhs_n<0 { 
            Err("[SHR] value must be natural number".to_string())
        } else {
            Ok(Value::Integer(lhs_n>>rhs_n))
        }
    }
}

impl std::ops::Div for Value {
    type Output = Result<Value, String>;
    fn div(self, rhs: Self) -> Self::Output {
        if let (&Value::Integer(lhs_n),&Value::Integer(rhs_n)) = (&self,&rhs) {
            if rhs_n == 0 {
                return Err("Division by 0".to_string());
            }
            Ok(Value::Integer(lhs_n / rhs_n))
        } else if let (&Value::Real(lhs_f),&Value::Real(rhs_f)) = (&self,&rhs) {
            if rhs_f == 0.0 {
                return Err("Division by 0".to_string());
            }
            Ok(Value::Real(lhs_f / rhs_f))
        } else if let &Value::Real(lhs_f) = &self {
            let rhs_f = if let &Value::Integer(rhs_n1) = &rhs {
                rhs_n1 as f64
            } else {
                return Err(format!("RHS has invaild type, expect float or integer, got: {:#?}", rhs));
            };
            if rhs_f == 0.0 {
                return Err("Division for 0".to_string());
            }

            Ok(Value::Real(lhs_f/rhs_f))
        }else if let &Value::Real(rhs_f) = &rhs {
            if rhs_f == 0.0 {
                return Err("Division for 0".to_string());
            }
            let lhs_f = if let &Value::Integer(lhs_n1) = &self {
                lhs_n1 as f64
            } else {
                return Err(format!("RHS has invaild type, expect float or integer, got: {:#?}", rhs));
            };

            Ok(Value::Real(lhs_f/rhs_f))
        }
        else {
            Err("Both value MUST Be integer,float or double.".to_string())
        }
    }
}

impl Value {
    pub fn new_obj(obj_name: String) -> Self {
        Self::Object(obj_name)
    }
    pub fn new_boolean_from(v: i64) -> Self {
        if v == 0 { Self::Boolean(false) }
        else { Self::Boolean(true) }
    }
    pub fn new(string: String) -> Self {
        // convert string to specified value
        let mut strtrim = string.trim();
        let radix = if strtrim.starts_with("0x") { 
            16 
        } 
        else if strtrim.starts_with("0b") {2}
        else if strtrim.starts_with("0o") {8}
        else {10};
        strtrim = if strtrim.starts_with("0x") || strtrim.starts_with("0b") || strtrim.starts_with("0o") {
            &strtrim[2..]
        } else {
            &strtrim[..]
        };

        if let Ok(v) = i128::from_str_radix(strtrim, radix) {
            Self::Integer(v)
        } else if let Ok(v)=strtrim.parse::<f64>() {
            Self::Real(v)
        } else if let Ok(v)=strtrim.parse::<bool>() {
            Self::Boolean(v)
        } else if let Ok(v)=strtrim.parse::<char>() {
            Self::Char(v)
        } else {
            if strtrim.len() == 0 {
                Self::Null
            } else {
                Self::Str(strtrim.to_string())
            }
        }
    }

    pub fn is_integer(self) -> bool {
        matches!(self, Value::Integer(_))
    }

    pub fn is_real(self) -> bool {
        matches!(self, Value::Real(_))
    }

    pub fn is_char(self) -> bool {
        matches!(self, Value::Char(_))
    }

    pub fn is_string(self) -> bool {
        matches!(self, Value::Str(_))
    }
    
    pub fn is_null(self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn to_datatype(self) -> DataType {
        match self {
            Value::Null => DataType::Unknown,
            Value::Integer(_) => DataType::I64,
            Value::Real(_) => DataType::F64,
            Value::Char(_) |  Value::Boolean(_)=> DataType::I8,
            _ => todo!()
        }
    }

    pub fn to_literal(self) -> Result<i128,String> {
        let Value::Integer(n) = self else {
            return Err("Cannot get integer from Value".to_string());
        };
        Ok(n)
    }

    pub fn to_real(self) -> Result<f64,String> {
        let Value::Real(n) = self else {
            return Err("Cannot get real number from Value".to_string());
        };
        Ok(n)
    }

    pub fn to_char(self) -> Result<char,String> {
        let Value::Char(n) = self else {
            return Err("Cannot get integer from Value".to_string());
        };
        Ok(n)
    }

    pub fn to_string(self) -> Result<String,String> {
        let Value::Str(n) = self else {
            return Err("Cannot get string from Value".to_string());
        };
        Ok(n)
    }
}
