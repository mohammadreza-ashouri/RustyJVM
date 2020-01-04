use super::class;
use super::native;
use super::super::model;
use super::sig;
use super::symref;
use super::constant_pool::{ConstantPool, ConstantPoolEntry};

use lib::Library;

use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug)]
pub struct ClassLoader {
    class_paths: Vec<PathBuf>,
    classes: HashMap<sig::Class, Rc<class::Class>>,

    natives: Vec<Rc<Library>>,
    unbound_natives: Vec<symref::Method>,
}

impl ClassLoader {
    pub fn new(class_paths: Vec<PathBuf>) -> ClassLoader {
        ClassLoader {
            class_paths: class_paths,
            classes: HashMap::new(),
            natives: Vec::new(),
            unbound_natives: Vec::new(),
        }
    }

    fn find_class_bytes(&self, name: &str) -> Result<Vec<u8>, &'static str> {
        for path in &self.class_paths {
            let path = {
                let mut path = path.clone();
                path.push(name);
                path.set_extension("class");
                path
            };

            if path.exists() {
                return File::open(path)
                    .and_then(|mut file| {
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf)?;
                        Ok(buf)
                    })
                    .or(Err("Could not load class"));
            }
        }
        Err("Class not found")
    }

    fn load_class_bytes(&mut self, sig: &sig::Class, bytes: Vec<u8>) -> Rc<class::Class> {
        let class = model::Class::new(bytes);
        self.load_class(sig, class)
    }

    fn load_class(&mut self, sig: &sig::Class, model: model::Class) -> Rc<class::Class> {
        let rcp = ConstantPool::new(&model.constant_pool);
        let sigs_match = {
            if let Some(ConstantPoolEntry::ClassRef(ref symref)) = rcp[model.this_class] {
				print!("{:?}", symref.sig);
                *sig == symref.sig
            } else {
                panic!("this_class({}) must point to a ClassRef", model.this_class);
            }
        };
        if sigs_match {
            let symref = symref::Class { sig: sig.clone() };
            let (class, unbound_natives) = class::Class::new(symref.clone(), None, rcp, model);

            for method in unbound_natives {
                let method_symref = symref::Method {
                    class: symref.clone(),
                    sig: method.clone(),
                };
                let lib = self.natives
                    .iter()
                    .find(|lib| native::has_method(&lib, &method_symref.clone()));
                if lib.is_some() {
                    class.bind_native_method(method, lib.unwrap().clone());
                } else {
                    self.unbound_natives.push(method_symref);
                }
            }

            let rc = Rc::new(class);
            self.classes.insert(sig.clone(), rc.clone());
            rc
        } else {
           panic!("Class signature mismatch: given {:?}", sig);
       }
        
    }

    pub fn load_library(&mut self, path: &str) {
        self.natives.push(Rc::new(native::load(path)));
        self.bind_native_methods();
    }

    pub fn bind_native_methods(&mut self) {
        let natives = self.natives.clone();
        let mut to_bind: HashMap<symref::Method, Rc<Library>> = HashMap::new();
        self.unbound_natives.retain(|method| {
            let lib = natives.iter().find(|lib| native::has_method(lib, method));
            if lib.is_some() {
                to_bind.insert(method.clone(), lib.unwrap().clone());
                true
            } else {
                false
            }
        });

        for (method, lib) in to_bind {
            let class = self.resolve_class(&method.class.sig);
            class.bind_native_method(method.sig, lib.clone());
        }
    }

    pub fn resolve_class(&mut self, sig: &sig::Class) -> Rc<class::Class> {
        if let Some(class) = self.classes.get(sig) {
            // the class has been resolved
            return class.clone();
        }

        match *sig {
            sig::Class::Scalar(ref name) => {
                let class_bytes = self.find_class_bytes(name)
                    .expect(&format!("could not load class {}", name));
                self.load_class_bytes(sig, class_bytes)
            }
            sig::Class::Array(ref component) => {
                Rc::new(class::Class::new_array(*component.clone()))
            }
        }
    }
}
