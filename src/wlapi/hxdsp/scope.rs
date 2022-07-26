// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexodsp::{Matrix, NodeId, ScopeHandle};
use hexotk::ScopeModel;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wlambda::*;

struct ScopeData {
    matrix: Arc<Mutex<Matrix>>,
    handle: Arc<ScopeHandle>,
    node_id: NodeId,
}

impl ScopeModel for ScopeData {
    fn signal_count(&self) -> usize {
        3
    }
    fn signal_len(&self) -> usize {
        self.handle.len()
    }
    fn get(&self, sig: usize, idx: usize) -> f32 {
        self.handle.read(sig, idx)
    }
    fn is_active(&self, sig: usize) -> bool {
        self.handle.is_active(sig)
    }
    fn fmt_val(&self, sig: usize, buf: &mut [u8]) -> usize {
        let mut max = -99999.0_f32;
        let mut min = 99999.0_f32;
        for i in 0..self.signal_len() {
            let v = self.handle.read(sig, i);
            max = max.max(v);
            min = min.min(v);
        }
        let rng = max - min;

        use std::io::Write;
        let max_len = buf.len();
        let mut bw = std::io::BufWriter::new(buf);
        match write!(
            bw,
            "in{} min: {:6.3} max: {:6.3} rng: {:6.3}",
            sig + 1,
            min,
            max,
            rng
        ) {
            Ok(_) => {
                if bw.buffer().len() > max_len {
                    max_len
                } else {
                    bw.buffer().len()
                }
            }
            Err(_) => 0,
        }
    }
}

#[derive(Clone)]
pub struct VScopeModel(Rc<RefCell<dyn ScopeModel>>);

impl VScopeModel {
    pub fn new(matrix: Arc<Mutex<Matrix>>, node_id: NodeId) -> Self {
        let handle = {
            let m = matrix.lock().expect("Matrix lockable");
            let handle = m.get_scope_handle(node_id.instance() as usize);
            if let Some(handle) = handle {
                handle
            } else {
                m.get_scope_handle(0).unwrap()
            }
        };

        Self(Rc::new(RefCell::new(ScopeData {
            matrix: matrix.clone(),
            handle,
            node_id: node_id.clone(),
        })))
    }
}

impl VValUserData for VScopeModel {
    fn s(&self) -> String {
        format!("$<UI::ScopeModel>")
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }

    fn call_method(&self, _key: &str, _env: &mut Env) -> Result<VVal, StackAction> {
        Ok(VVal::None)
    }
}

pub fn vv2scope_model(mut v: VVal) -> Option<Rc<RefCell<dyn ScopeModel>>> {
    v.with_usr_ref(|model: &mut VScopeModel| model.0.clone())
}
