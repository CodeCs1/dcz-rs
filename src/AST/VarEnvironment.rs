use std::{collections::HashMap, ops::{Deref, Index, IndexMut}};
use crate::AST::expr_node::{Variable};

type VarList = HashMap<String, Variable>;
type VariableResult<'env> = Result<&'env Variable,String>;
#[derive(Clone,Debug)]
pub struct VarEnvironment{
    last_save: Option<Box<VarEnvironment>>,
    enclosing: Option<Box<VarEnvironment>>,
    variables: VarList,
}

impl VarEnvironment {
    pub fn new() -> Self {
        Self {
            last_save: None,
            enclosing: None,
            variables: VarList::new()
        }
    }
    pub fn new_with_enclosing(enclosing: VarEnvironment) -> Self {
        Self { enclosing: Some(
            Box::new(enclosing)
        ), variables: VarList::new(),
            last_save: None
        }
    }
    pub fn save(&mut self) {
        if self.last_save.is_none() {
            self.last_save = Some(
                Box::new(Self {
                    enclosing: self.enclosing.clone(),
                    last_save: self.last_save.clone(),
                    variables: self.variables.clone()
                })
            );
        }
    }
    pub fn load(&mut self,new_varenv: VarEnvironment) {
        self.last_save = Some(Box::new(self.deref().clone()));
        self.enclosing = new_varenv.enclosing;
        self.variables = new_varenv.variables;
    }
    pub fn load_last(&mut self) {
        let Some(last_env) = self.last_save.clone() else {
            return;
        };
        self.enclosing = last_env.enclosing;
        self.last_save = None;
        self.variables = last_env.variables;
    }
    pub fn get(&self, name: &str) -> VariableResult<'_>{
        let Some(v) = self.variables.get(name) else {
            return Err(format!("Variable '{}' not defined",name));
        };
        if let Some(varenv) = &self.enclosing {
            return varenv.get(name);
        }
        Ok(v)
    }
    pub fn contains(&self, name: &String) -> bool {
        return self.variables.contains_key(name)
    }
    pub fn new_value(&mut self, name: String, val: Variable) {
        self.variables.insert(name, val);
    }
    pub fn get_mut(&mut self, name: &str) -> Result<&mut Variable,String> {
        let Some(var) = self.variables.get_mut(name) else {
            return Err(format!("Undefined variable '{}'", name));
        };
        Ok(var)
    }
    // /// Return error as Some of String
    // pub fn assign(&mut self, name: &str, val: Variable) -> Option<String> {
    //     let Ok(var) =self.get_mut("") else {
    //         return Some(format!("Undefined variable '{}'", name))
    //     };
    //     *var = val;
    //     None
    // }
}

impl Index<&str> for VarEnvironment {
    type Output = Variable;
    fn index(&self, index: &str) -> &Self::Output {
        let Ok(v) = self.get(index) else {
            panic!("Variable '{}' not exist",index);
        };
        v
    } 
}

impl IndexMut<&str> for VarEnvironment {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        let Ok(v) = self.get_mut(index) else {
            panic!("Variable '{}' not exist", index);
        };
        v
    }
}