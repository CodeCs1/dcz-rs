use object::{write::{Object, Relocation, Symbol, SymbolId}, RelocationKind};

use crate::Value::Value;



pub struct ObjectOut<'a> {
    obj: Object<'a>
}

impl<'a> ObjectOut<'a> {
    pub fn new() -> Self {
        Self { 
            obj: Object::new(object::BinaryFormat::Elf, object::Architecture::X86_64, object::Endianness::Little)
        }
    }

    /// add_func(name, opcode)
    /// 
    /// Add function name into .text section 
    ///

    pub fn add_func(&mut self, name: &str, opcode: Vec<u8>) {
         //let text_sect = self.obj.add_section(self.obj.segment_name(object::write::StandardSegment::Text).to_vec(),".text".as_bytes().to_vec(), object::SectionKind::Text);
         let text_sect = self.obj.section_id(object::write::StandardSection::Text);
         self.obj.append_section_data(text_sect, &opcode, 1);
         self.obj.add_symbol(Symbol { 
             name: name.as_bytes().to_vec(),
             value: 0,
             size: opcode.len() as u64,
             kind: object::SymbolKind::Text,
             scope: object::SymbolScope::Linkage,
             weak: false,
             section: object::write::SymbolSection::Section(text_sect),
             flags: object::SymbolFlags::None });
    }

    ///
    /// add_str_data(String, String)
    ///
    /// Return string offset (see add_reloc())
    pub fn add_str_data(&mut self, name: String, data: String) -> SymbolId {
        let data_sect=self.obj.section_id(object::write::StandardSection::Data);

        let str_offset = self.obj.append_section_data(data_sect, data.as_bytes(), 1);
        self.obj.add_symbol(Symbol {
            name: name.as_bytes().to_vec(),
            value: str_offset,
            size: data.len() as u64,
            kind: object::SymbolKind::Data,
            scope: object::SymbolScope::Linkage,
            weak: false,
            section: object::write::SymbolSection::Section(data_sect),
            flags: object::SymbolFlags::None
        })
    }

    ///
    /// add_value_data(String, Value)
    ///
    /// Return SymbolID (see add_reloc())
    pub fn add_value_data(&mut self, name: String, data: Value) -> SymbolId {
        let data_sect = self.obj.section_id(object::write::StandardSection::Data);

        let offset = self.obj.append_section_data(data_sect, &data.clone().to_literal().to_ne_bytes().to_vec() as &[u8], 1);
        self.obj.add_symbol(Symbol {
            name: name.as_bytes().to_vec(),
            value: offset,
            size: (data.to_literal().checked_ilog10().unwrap_or(0)+1) as u64,
            kind: object::SymbolKind::Data,
            scope: object::SymbolScope::Linkage,
            weak: false,
            section: object::write::SymbolSection::Section(data_sect),
            flags: object::SymbolFlags::None
        })
    }

    ///
    /// add_reloc(SymbolId)
    ///
    ///
    pub fn add_reloc(&mut self, sym: SymbolId, at: u64) {
        let tsect = self.obj.section_id(object::write::StandardSection::Text);
        self.obj.add_relocation(tsect, Relocation {
            offset: at,
            symbol: sym,
            addend: 0,
            flags: object::RelocationFlags::Generic { kind: RelocationKind::Absolute, encoding:object::RelocationEncoding::Generic, size: 64 } 
        }).expect("Failed to relocation");
    }

    pub fn write_buff(&self) -> Vec<u8> {
        self.obj.write().expect("Failed to write ELF 2 buffer")
    }


}
