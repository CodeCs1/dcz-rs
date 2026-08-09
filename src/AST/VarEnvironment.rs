use std::{collections::HashMap, marker::PhantomData};
use crate::AST::expr_node::Variable;
type VarList = HashMap<String, Variable>;

#[derive(Debug,Clone)]
pub struct VariableTable<'a> {
    parent: Option< Box< VariableTable<'a> > >,
    vars: VarList,
    _phantomdata: PhantomData<&'a VariableTable<'a>>
}

impl<'a> VariableTable<'a> {
    pub fn new(parent: Option<Box<VariableTable<'a>>> ) -> Self {
        Self {
            parent,
            vars: HashMap::new(),
            _phantomdata: PhantomData
        }
    }
    pub fn find(&mut self, vars_name: &String) -> Option<Variable> {
        let Some(var) = self.vars.iter().find(|v| v.0 == vars_name).and_then(|(_,o)| Some(o.clone())) else {
            let Some(parent) = &mut self.parent else {
                return None;
            };
            return parent.clone().find(vars_name);
        };
        Some(var)

    }
    pub fn add_new_var(&mut self, var: &Variable) {
        self.vars.insert(var.name.to_owned(), var.clone());
    }
}

#[derive(Clone,Debug)]
pub struct VariableSymbolTableRoot<'a> {
    global: VariableTable<'a>,
    current_table: Option<VariableTable<'a>>
}

impl <'a> VariableSymbolTableRoot<'a> {
    pub fn new() -> Self {
        Self {
            global: VariableTable::new(None),
            current_table: None
        }
    }
    pub fn find(&mut self, vars_name: &String) -> Option<Variable> {
        self.local_or_global().find(vars_name)
    }

    pub fn add_vars(&mut self, vars: Variable) {
        self.local_or_global().add_new_var(&vars);
    }
    fn local_or_global(&mut self) -> &mut VariableTable<'a> {
        let Some(current_table) = self.current_table.as_mut() else {
            return &mut self.global
        };
        current_table
    }
    pub fn clear_local_table(&mut self) {
        let Some(current_table) = self.current_table.as_mut() else {
            return;
        };
        let Some(parents) = current_table.parent.as_mut() else {
            self.current_table = None;
            return;
        };
        *current_table = *parents.clone();
    }
    pub fn add_local_table(&mut self) {
        self.current_table = Some(
            VariableTable::new(
                Some(Box::new(self.local_or_global().clone()))
            )
        )
    }
}
