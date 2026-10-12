// Additional native tables for java.lang

use crate::vm::native::*;

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];

// java.lang.Iterable host shim
pub(crate) fn iterable_iterator(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::List(list)) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let iter_id = alloc(vm, "Ljava/util/Iterator;", Native::Iter(IterKind::List { list: list.clone(), index: 0 }))?;
    Ok(JValue::Obj(iter_id))
}

pub(crate) const ITERABLE_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/Iterable;",
        "iterator",
        "()Ljava/util/Iterator;",
        true,
        iterable_iterator
    ),
];

// java.lang.annotation.Annotation host shim - minimal implementation
pub(crate) const ANNOTATION_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/lang/annotation/Annotation;",
        "annotationType",
        "()Ljava/lang/Class;",
        true,
        |vm, args| {
            let Some(Native::Obj(id)) = payload(vm, args[0]) else {
                return Err(npe(vm));
            };
            let class = vm.arena.objects[id as usize].class;
            Ok(JValue::Obj(vm.arena.alloc(class, Vec::new(), Some(Native::ClassObj(ClassOrPrim::Class(class)))))
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "equals",
        "(Ljava/lang/Object;)Z",
        true,
        |vm, args| {
            let a = payload(vm, args[0]);
            let b = payload(vm, args[1]);
            match (a, b) {
                (Some(a), Some(b)) => Ok(JValue::Int(i32::from(std::ptr::eq(a, b)))),
                _ => Ok(JValue::Int(0)),
            }
        }
    ),
    ne(
        "Ljava/lang/annotation/Annotation;",
        "hashCode",
        "()I",
        true,
        |vm, args| {
            let id = match args[0] {
                JValue::Obj(id) => id,
                _ => return Err(npe(vm)),
            };
            Ok(JValue::Int(id as i32))
        }
    ),
];

// java.util.Map$Entry host shim
pub(crate) fn map_entry_get_key(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].0)
    } else {
        Err(npe(vm))
    }
}

pub(crate) fn map_entry_get_value(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::MapEntry { map, idx }) = payload(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some(Native::Map(entries)) = payload(vm, JValue::Obj(*map)) else {
        return Err(npe(vm));
    };
    if *idx < entries.len() {
        Ok(entries[*idx].1)
    } else {
        Err(npe(vm))
    }
}

pub(crate) const MAP_ENTRY_TABLE: &[NativeEntry] = &[
    ne(
        "Ljava/util/Map$Entry;",
        "getKey",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_key
    ),
    ne(
        "Ljava/util/Map$Entry;",
        "getValue",
        "()Ljava/lang/Object;",
        true,
        map_entry_get_value
    ),
];
ENDOFFILE

cat /tmp/lang_additions.rs >> /home/volkor/git/dexvm/src/vm/native/java/lang/enum.rs
