//use crate::id::Id;
use {
    std::collections::{HashMap, HashSet},
    makepad_id_macros::*,
    crate::{
        liveerror::{LiveError, LiveFileError},
        liveparser::LiveParser,
        livedocument::LiveDocument,
        livenode::{LiveNode, LiveValue, LiveType, LiveTypeInfo, LiveNodeSlice},
        liveid::{LiveId, LiveFileId, LivePtr, LiveModuleId},
        token::TokenId,
        span::Span,
        lex::lex,
        liveexpander::{LiveExpander, ScopeStack}
    }
};


pub struct LiveFile {
    pub module_id: LiveModuleId,
    pub line_offset: usize,
    pub file_name: String,
    pub source: String,
    pub document: LiveDocument,
}

#[derive(Default)]
pub struct LiveRegistry {
    pub file_ids: HashMap<String, LiveFileId>,
    pub module_id_to_file_id: HashMap<LiveModuleId, LiveFileId>,
    pub live_files: Vec<LiveFile>,
    pub live_type_infos: HashMap<LiveType, LiveTypeInfo>,
    pub dep_order: Vec<(LiveModuleId, TokenId)>,
    pub dep_graph: HashMap<LiveModuleId, HashSet<LiveModuleId >>, // this contains all the dependencies a crate has
    pub expanded: Vec<LiveDocument >,
}

pub struct LiveDocNodes<'a> {
    pub nodes: &'a [LiveNode],
    pub file_id: LiveFileId,
    pub index: usize
}

impl LiveRegistry {
    pub fn ptr_to_node(&self, live_ptr: LivePtr) -> &LiveNode {
        let doc = &self.expanded[live_ptr.file_id.to_index()];
        &doc.resolve_ptr(live_ptr.index as usize)
    }
    
    pub fn file_id_to_file_name(&self, file_id: LiveFileId) -> &str {
        &self.live_files[file_id.to_index()].file_name
    }
    
    pub fn ptr_to_doc_node(&self, live_ptr: LivePtr) -> (&LiveDocument, &LiveNode) {
        let doc = &self.expanded[live_ptr.file_id.to_index()];
        (doc, &doc.resolve_ptr(live_ptr.index as usize))
    }
    
    pub fn ptr_to_doc(&self, live_ptr: LivePtr) -> &LiveDocument {
        &self.expanded[live_ptr.file_id.to_index()]
    }
    
    pub fn file_id_to_doc(&self, file_id: LiveFileId) -> &LiveDocument {
        &self.expanded[file_id.to_index()]
    }
    
    pub fn ptr_to_nodes_index(&self, live_ptr: LivePtr) -> (&[LiveNode], usize) {
        let doc = &self.expanded[live_ptr.file_id.to_index()];
        (&doc.nodes, live_ptr.index as usize)
    }
    
    pub fn token_id_to_origin_doc(&self, token_id: TokenId) -> &LiveDocument {
        &self.live_files[token_id.file_id().to_index()].document
    }
    
    pub fn token_id_to_expanded_doc(&self, token_id: TokenId) -> &LiveDocument {
        &self.expanded[token_id.file_id().to_index()]
    }
    /*
    pub fn module_path_str_id_to_doc(&self, module_path: &str, id:Id) -> Option<LiveDocNodes> {
        self.module_path_id_to_doc(ModulePath::from_str(module_path).unwrap(), id)
    }
    */
    pub fn module_path_id_to_doc(&self, module_id: LiveModuleId, id: LiveId) -> Option<LiveDocNodes> {
        if let Some(file_id) = self.module_id_to_file_id.get(&module_id) {
            let doc = &self.expanded[file_id.to_index()];
            if id != LiveId::empty() {
                if doc.nodes.len() == 0 {
                    println!("module_path_id_to_doc zero nodelen {}", self.file_id_to_file_name(*file_id));
                    return None
                }
                if let Some(index) = doc.nodes.child_by_name(0, id) {
                    return Some(LiveDocNodes {nodes: &doc.nodes, file_id: *file_id, index});
                }
                else {
                    return None
                }
            }
            else {
                return Some(LiveDocNodes {nodes: &doc.nodes, file_id: *file_id, index: 0});
            }
        }
        None
    }
    
    pub fn find_scope_item_via_class_parent(&self, start_ptr: LivePtr, item: LiveId) -> Option<(&[LiveNode], usize)> {
        let (nodes, index) = self.ptr_to_nodes_index(start_ptr);
        if let LiveValue::Class {class_parent, ..} = &nodes[index].value {
            // ok its a class so now first scan up from here.
            
            if let Some(index) = nodes.scope_up_by_name(index, item) {
                // item can be a 'use' as well.
                // if its a use we need to resolve it, otherwise w found it
                if let LiveValue::Use(module_path) = &nodes[index].value {
                    if let Some(ldn) = self.module_path_id_to_doc(*module_path, nodes[index].id) {
                        return Some((ldn.nodes, ldn.index))
                    }
                }
                else {
                    return Some((nodes, index))
                }
            }
            else {
                if let Some(class_parent) = class_parent {
                    if class_parent.file_id != start_ptr.file_id {
                        return self.find_scope_item_via_class_parent(*class_parent, item)
                    }
                }
                
            }
        }
        else {
            println!("WRONG TYPE  {:?}", nodes[index].value);
        }
        None
    }
    
    
    pub fn live_error_to_live_file_error(&self, live_error: LiveError) -> LiveFileError {
        let live_file = &self.live_files[live_error.span.file_id().to_index()];
        live_error.to_live_file_error(&live_file.file_name, &live_file.source, live_file.line_offset)
    }
    
    
    pub fn token_id_to_span(&self, token_id: TokenId) -> Span {
        self.live_files[token_id.file_id().to_index()].document.token_id_to_span(token_id)
    }
    
    pub fn insert_dep_order(&mut self, module_id: LiveModuleId, token_id: TokenId, own_module_id: LiveModuleId) {
        let self_index = self.dep_order.iter().position( | v | v.0 == own_module_id).unwrap();
        if let Some(other_index) = self.dep_order.iter().position( | v | v.0 == module_id) {
            if other_index > self_index {
                self.dep_order.remove(other_index);
                self.dep_order.insert(self_index, (module_id, token_id));
            }
        }
        else {
            self.dep_order.insert(self_index, (module_id, token_id));
        }
    }
    
    pub fn parse_live_file(
        &mut self,
        file_name: &str,
        own_module_id: LiveModuleId,
        source: String,
        live_type_infos: Vec<LiveTypeInfo>,
        line_offset: usize
    ) -> Result<LiveFileId, LiveFileError> {
        
        // lets register our live_type_infos
        
        let (is_new_file_id, file_id) = if let Some(file_id) = self.file_ids.get(file_name) {
            (false, *file_id)
        }
        else {
            let file_id = LiveFileId::index(self.live_files.len());
            (true, file_id)
        };
        
        let lex_result = match lex(source.chars(), file_id) {
            Err(msg) => return Err(msg.to_live_file_error(file_name, &source, line_offset)), //panic!("Lex error {}", msg),
            Ok(lex_result) => lex_result
        };
        
        let mut parser = LiveParser::new(&lex_result.tokens, &live_type_infos, file_id);
        
        let mut document = match parser.parse_live_document() {
            Err(msg) => return Err(msg.to_live_file_error(file_name, &source, line_offset)), //panic!("Parse error {}", msg.to_live_file_error(file, &source)),
            Ok(ld) => ld
        };
        document.strings = lex_result.strings;
        document.tokens = lex_result.tokens;
        
        // update our live type info
        for live_type_info in live_type_infos {
            if let Some(info) = self.live_type_infos.get(&live_type_info.live_type) {
                if info.module_id != live_type_info.module_id ||
                info.live_type != live_type_info.live_type {
                    panic!()
                }
            };
            self.live_type_infos.insert(live_type_info.live_type, live_type_info);
        }
        
        // let own_crate_module = CrateModule(crate_id, module_id);
        
        if self.dep_order.iter().position( | v | v.0 == own_module_id).is_none() {
            self.dep_order.push((own_module_id, TokenId::new(file_id, 0)));
        }
        else {
            // marks dependencies dirty recursively (removes the expanded version)
            fn mark_dirty(mp: LiveModuleId, registry: &mut LiveRegistry) {
                if let Some(id) = registry.module_id_to_file_id.get(&mp) {
                    registry.expanded[id.to_index()].recompile = true;
                }
                //registry.expanded.remove(&cm);
                
                let mut dirty = Vec::new();
                for (mp_iter, hs) in &registry.dep_graph {
                    if hs.contains(&mp) { // this
                        dirty.push(*mp_iter);
                    }
                }
                for d in dirty {
                    mark_dirty(d, registry);
                }
            }
            mark_dirty(own_module_id, self);
        }
        
        let mut dep_graph_set = HashSet::new();
        
        for node in &mut document.nodes {
            match &mut node.value {
                LiveValue::Use(module_id) => {
                    if module_id.0 == id!(crate) { // patch up crate refs
                        module_id.0 = own_module_id.0
                    };
                    
                    dep_graph_set.insert(*module_id);
                    self.insert_dep_order(*module_id, node.token_id.unwrap(), own_module_id);
                    
                }, // import
                LiveValue::Class {live_type, ..} => { // hold up. this is always own_module_path
                    let infos = self.live_type_infos.get(&live_type).unwrap();
                    for sub_type in infos.fields.clone() {
                        let sub_module_id = sub_type.live_type_info.module_id;
                        if sub_module_id != own_module_id {
                            dep_graph_set.insert(sub_module_id);
                            self.insert_dep_order(sub_module_id, node.token_id.unwrap(), own_module_id);
                        }
                    }
                }
                _ => {
                }
            }
        }
        self.dep_graph.insert(own_module_id, dep_graph_set);
        
        let live_file = LiveFile {
            module_id: own_module_id,
            file_name: file_name.to_string(),
            line_offset,
            source,
            document
        };
        self.module_id_to_file_id.insert(own_module_id, file_id);
        
        if is_new_file_id {
            self.file_ids.insert(file_name.to_string(), file_id);
            self.live_files.push(live_file);
            self.expanded.push(LiveDocument::new());
        }
        else {
            self.live_files[file_id.to_index()] = live_file;
            self.expanded[file_id.to_index()].recompile = true;
        }
        
        return Ok(file_id)
    }
    
    pub fn expand_all_documents(&mut self, errors: &mut Vec<LiveError>) {
        for (crate_module, _token_id) in &self.dep_order {
            let file_id = if let Some(file_id) = self.module_id_to_file_id.get(crate_module) {
                file_id
            }
            else {
                // ok so we have a token_id. now what.
                /*errors.push(LiveError {
                    origin: live_error_origin!(),
                    span: self.token_id_to_span(*token_id),
                    message: format!("Cannot find dependency: {}::{}", crate_module.0, crate_module.1)
                });*/
                continue
            };
            
            if !self.expanded[file_id.to_index()].recompile {
                continue;
            }
            let live_file = &self.live_files[file_id.to_index()];
            let in_doc = &live_file.document;
            
            let mut out_doc = LiveDocument::new();
            std::mem::swap(&mut out_doc, &mut self.expanded[file_id.to_index()]);
            out_doc.restart_from(&in_doc);
            
            let mut scope_stack = ScopeStack {
                stack: vec![Vec::new()]
            };
            //let len = in_doc.nodes[0].len();
            
            let mut live_document_expander = LiveExpander {
                live_registry: self,
                in_crate: crate_module.0,
                in_file_id: *file_id,
                scope_stack: &mut scope_stack,
                errors
            };
            // OK now what. how will we do this.
            live_document_expander.expand(in_doc, &mut out_doc);
            
            
            out_doc.recompile = false;
            
            std::mem::swap(&mut out_doc, &mut self.expanded[file_id.to_index()]);
        }
    }
}

