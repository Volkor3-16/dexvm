//! Gap Analyzer: Static analysis of extension DEX files to determine
//! exactly what external references are needed vs what the VM provides.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use dexvm::dex::DexFile;
use dexvm::dex::insn::{decode_all, Insn, InvokeKind as DexInvokeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum InvokeKind {
    Virtual,
    Static,
    Direct,
    Super,
    Interface,
}

impl From<DexInvokeKind> for InvokeKind {
    fn from(k: DexInvokeKind) -> Self {
        match k {
            DexInvokeKind::Virtual => InvokeKind::Virtual,
            DexInvokeKind::Super => InvokeKind::Super,
            DexInvokeKind::Direct => InvokeKind::Direct,
            DexInvokeKind::Static => InvokeKind::Static,
            DexInvokeKind::Interface => InvokeKind::Interface,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ExtClassRef {
    descriptor: String,
    kind: ClassRefKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ClassRefKind {
    Superclass,
    Interface,
    NewInstance,
    CheckCast,
    InstanceOf,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ExtMethodRef {
    class: String,
    name: String,
    signature: String,
    is_static: bool,
    invoke_kind: InvokeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ExtFieldRef {
    class: String,
    name: String,
    descriptor: String,
    is_static: bool,
    access_kind: FieldAccessKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum FieldAccessKind {
    Iget,
    Iput,
    Sget,
    Sput,
}

impl std::fmt::Display for FieldAccessKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Default)]
struct ReferenceSet {
    classes: BTreeSet<ExtClassRef>,
    methods: BTreeSet<ExtMethodRef>,
    fields: BTreeSet<ExtFieldRef>,
}

impl ReferenceSet {
    fn merge(&mut self, other: &ReferenceSet) {
        self.classes.extend(other.classes.iter().cloned());
        self.methods.extend(other.methods.iter().cloned());
        self.fields.extend(other.fields.iter().cloned());
    }
}

fn analyze_dex(dex: &DexFile) -> ReferenceSet {
    let mut refs = ReferenceSet::default();

    // What this DEX defines internally (not external)
    let defined_classes: BTreeSet<String> = dex.classes.iter()
        .map(|c| dex.type_descriptor(c.class_idx).to_string())
        .collect();

    for class in &dex.classes {
        let class_desc = dex.type_descriptor(class.class_idx).to_string();
        let Some(class_data) = &class.class_data else { continue };

        // Superclass
        if class.superclass_idx != u32::MAX {
            let super_desc = dex.type_descriptor(class.superclass_idx).to_string();
            if !defined_classes.contains(&super_desc) {
                refs.classes.insert(ExtClassRef { descriptor: super_desc, kind: ClassRefKind::Superclass });
            }
        }

        // Interfaces
        for &iface_idx in &class.interfaces {
            let iface_desc = dex.type_descriptor(iface_idx).to_string();
            if !defined_classes.contains(&iface_desc) {
                refs.classes.insert(ExtClassRef { descriptor: iface_desc, kind: ClassRefKind::Interface });
            }
        }

        for encoded in class_data.direct_methods.iter().chain(&class_data.virtual_methods) {
            let method = &dex.methods[encoded.method_idx as usize];
            let method_name = dex.strings[method.name as usize].as_ref();
            let proto = &dex.protos[method.proto as usize];
            let sig = format!("({}){}",
                proto.params.iter().map(|p| dex.type_descriptor(*p)).collect::<String>(),
                dex.type_descriptor(proto.return_type));
            let is_static = (encoded.access_flags & 0x0008) != 0;

            if let Some(code) = &encoded.code {
                if let Ok(decoded) = decode_all(&code.insns) {
                    for insn in decoded.insns.iter() {
                        match insn {
                            Insn::Invoke(ref invoke_kind, ref method_idx, _args) => {
                                let m = &dex.methods[*method_idx as usize];
                                let owner = dex.type_descriptor(dex.types[m.class as usize]).to_string();
                                let name = dex.strings[m.name as usize].as_ref();
                                let proto = &dex.protos[m.proto as usize];
                                let sig = format!("({}){}",
                                    proto.params.iter().map(|p| dex.type_descriptor(*p)).collect::<String>(),
                                    dex.type_descriptor(proto.return_type));
                                let invoke_kind_local: InvokeKind = (*invoke_kind).into();
                                let is_static = matches!(invoke_kind_local, InvokeKind::Static);
                                if !defined_classes.contains(&owner) {
                                    refs.methods.insert(ExtMethodRef {
                                        class: owner, name: name.to_string(), signature: sig,
                                        is_static, invoke_kind: invoke_kind_local,
                                    });
                                }
                            }
                            Insn::NewInstance(_reg, ref type_idx) => {
                                let desc = dex.type_descriptor(*type_idx).to_string();
                                if !defined_classes.contains(&desc) {
                                    refs.classes.insert(ExtClassRef { descriptor: desc, kind: ClassRefKind::NewInstance });
                                }
                            }
                            Insn::CheckCast(_reg, ref type_idx) => {
                                let desc = dex.type_descriptor(*type_idx).to_string();
                                if !defined_classes.contains(&desc) {
                                    refs.classes.insert(ExtClassRef { descriptor: desc, kind: ClassRefKind::CheckCast });
                                }
                            }
                            Insn::InstanceOf(_reg, _reg2, ref type_idx) => {
                                let desc = dex.type_descriptor(*type_idx).to_string();
                                if !defined_classes.contains(&desc) {
                                    refs.classes.insert(ExtClassRef { descriptor: desc, kind: ClassRefKind::InstanceOf });
                                }
                            }
                            Insn::IGet(_reg, _reg2, ref field_idx) | Insn::IPut(_reg, _reg2, ref field_idx) |
                            Insn::IGetObj(_reg, _reg2, ref field_idx) | Insn::IPutObj(_reg, _reg2, ref field_idx) => {
                                let f = &dex.fields[*field_idx as usize];
                                let owner = dex.type_descriptor(dex.types[f.class as usize]).to_string();
                                let name = dex.strings[f.name as usize].as_ref();
                                let desc = dex.type_descriptor(f.ty).to_string();
                                let is_static = false;
                                let kind = if matches!(insn, Insn::IGet(_, _, _) | Insn::IGetObj(_, _, _)) { FieldAccessKind::Iget } else { FieldAccessKind::Iput };
                                if !defined_classes.contains(&owner) {
                                    refs.fields.insert(ExtFieldRef { class: owner, name: name.to_string(), descriptor: desc, is_static, access_kind: kind });
                                }
                            }
                            Insn::SGet(_reg, ref field_idx) | Insn::SPut(_reg, ref field_idx) |
                            Insn::SGetObj(_reg, ref field_idx) | Insn::SPutObj(_reg, ref field_idx) => {
                                let f = &dex.fields[*field_idx as usize];
                                let owner = dex.type_descriptor(dex.types[f.class as usize]).to_string();
                                let name = dex.strings[f.name as usize].as_ref();
                                let desc = dex.type_descriptor(f.ty).to_string();
                                let is_static = true;
                                let kind = if matches!(insn, Insn::SGet(_, _) | Insn::SGetObj(_, _)) { FieldAccessKind::Sget } else { FieldAccessKind::Sput };
                                if !defined_classes.contains(&owner) {
                                    refs.fields.insert(ExtFieldRef { class: owner, name: name.to_string(), descriptor: desc, is_static, access_kind: kind });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    refs
}

/// What the VM currently provides (shims + built-ins)
fn provided_by_vm() -> ReferenceSet {
    let mut refs = ReferenceSet::default();

    // Java built-ins (always available)
    let java_builtins = [
        "Ljava/lang/Object;", "Ljava/lang/String;", "Ljava/lang/Class;",
        "Ljava/lang/Throwable;", "Ljava/lang/Exception;", "Ljava/lang/RuntimeException;",
        "Ljava/lang/NullPointerException;", "Ljava/lang/IllegalArgumentException;",
        "Ljava/lang/IllegalStateException;", "Ljava/lang/UnsupportedOperationException;",
        "Ljava/lang/Boolean;", "Ljava/lang/Byte;", "Ljava/lang/Character;",
        "Ljava/lang/Short;", "Ljava/lang/Integer;", "Ljava/lang/Long;",
        "Ljava/lang/Float;", "Ljava/lang/Double;",
        "Ljava/lang/System;", "Ljava/lang/Runtime;",
        "Ljava/lang/StringBuilder;", "Ljava/lang/StringBuffer;",
        "Ljava/util/List;", "Ljava/util/ArrayList;", "Ljava/util/HashMap;",
        "Ljava/util/HashSet;", "Ljava/util/Map;", "Ljava/util/Set;",
        "Ljava/util/Collection;", "Ljava/util/Iterator;", "Ljava/util/Collections;",
        "Ljava/io/InputStream;", "Ljava/io/OutputStream;", "Ljava/io/Closeable;",
        "Ljava/io/Serializable;", "Ljava/lang/Comparable;", "Ljava/lang/Cloneable;",
        "Ljava/lang/Runnable;", "Ljava/lang/Thread;",
        "Ljava/lang/reflect/Type;", "Ljava/lang/reflect/Method;", "Ljava/lang/reflect/Field;",
        "Ljava/lang/reflect/Constructor;", "Ljava/lang/reflect/Modifier;",
        "Ljava/security/KeyPairGenerator;", "Ljava/security/MessageDigest;",
        "Ljava/text/SimpleDateFormat;", "Ljava/text/DateFormat;",
        "Ljava/util/Locale;", "Ljava/util/TimeZone;", "Ljava/util/Calendar;",
        "Ljava/util/Date;", "Ljava/util/Random;", "Ljava/util/UUID;",
        "Ljava/util/regex/Pattern;", "Ljava/util/regex/Matcher;",
        "Ljava/time/Instant;", "Ljava/time/LocalDate;", "Ljava/time/LocalDateTime;",
        "Ljava/time/ZoneId;", "Ljava/time/ZoneOffset;", "Ljava/time/format/DateTimeFormatter;",
        "Landroid/content/Context;", "Landroid/content/SharedPreferences;",
        "Landroid/preference/PreferenceManager;", "Landroid/app/Application;",
        "Landroid/os/Build;", "Landroid/os/Bundle;", "Landroid/util/Log;",
        "Landroid/webkit/WebView;", "Landroid/webkit/WebResourceRequest;",
        "Landroid/webkit/WebResourceResponse;",
        "Lokhttp3/OkHttpClient;", "Lokhttp3/Request;", "Lokhttp3/Response;",
        "Lokhttp3/RequestBody;", "Lokhttp3/FormBody;", "Lokhttp3/MultipartBody;",
        "Lokhttp3/Headers;", "Lokhttp3/Headers$Builder;", "Lokhttp3/MediaType;",
        "Lokio/Buffer;", "Lokio/BufferedSource;", "Lokio/ByteString;",
        "Lokio/Okio;", "Lokio/Source;",
        "Lorg/jsoup/Jsoup;", "Lorg/jsoup/nodes/Document;", "Lorg/jsoup/nodes/Element;",
        "Lorg/jsoup/select/Elements;", "Lorg/jsoup/select/Selector;",
        "Lkotlin/Unit;", "Lkotlin/jvm/functions/Function0;", "Lkotlin/jvm/functions/Function1;",
        "Lkotlin/jvm/functions/Function2;", "Lkotlin/jvm/internal/Ref$ObjectRef;",
        "Lkotlin/jvm/internal/Ref$IntRef;", "Lkotlin/jvm/internal/Ref$BooleanRef;",
        "Lkotlin/ranges/IntRange;", "Lkotlin/ranges/IntProgression;",
        "Lkotlin/collections/ArraysKt;", "Lkotlin/collections/CollectionsKt;",
        "Lkotlin/sequences/SequencesKt;", "Lkotlin/random/Random;",
        "Lkotlin/text/StringsKt;", "Lkotlin/io/CloseableKt;",
        "Lkotlin/coroutines/Continuation;", "Lkotlin/coroutines/CoroutineContext;",
        "Lkotlin/coroutines/jvm/internal/ContinuationImpl;",
        "Lkotlin/coroutines/jvm/internal/SuspendLambda;",
        "Lkotlin/coroutines/jvm/internal/BaseContinuationImpl;",
        "Lkotlinx/serialization/SerializationStrategy;",
        "Lkotlinx/serialization/DeserializationStrategy;",
        "Lkotlinx/serialization/KSerializer;",
        "Lkotlinx/serialization/json/Json;", "Lkotlinx/serialization/json/JsonElement;",
        "Lkotlinx/serialization/json/JsonObject;", "Lkotlinx/serialization/json/JsonArray;",
        "Lkotlinx/serialization/json/JsonPrimitive;", "Lkotlinx/serialization/json/JsonNull;",
        "Lkotlinx/serialization/json/JsonDecoder;", "Lkotlinx/serialization/json/JsonEncoder;",
        "Lkotlinx/serialization/json/JsonConfiguration;", "Lkotlinx/serialization/json/JsonBuilder;",
        "Lkotlinx/serialization/modules/SerializersModule;",
        "Lkotlinx/serialization/encoding/Decoder;", "Lkotlinx/serialization/encoding/Encoder;",
        "Lkotlinx/serialization/encoding/CompositeDecoder;",
        "Lkotlinx/serialization/descriptors/SerialDescriptor;",
        "Lkotlinx/serialization/internal/PluginGeneratedSerialDescriptor;",
        "Lkotlinx/serialization/json/internal/StreamingJsonEncoder;",
        "Lkotlinx/serialization/json/okio/OkioStreamsKt;",
        "Luy/kohesive/injekt/InjektKt;", "Luy/kohesive/injekt/api/InjektScope;",
        "Luy/kohesive/injekt/api/InjektFactory;", "Luy/kohesive/injekt/api/FullTypeReference;",
        "Leu/kanade/tachiyomi/source/online/HttpSource;",
        "Leu/kanade/tachiyomi/source/Source;",
        "Leu/kanade/tachiyomi/source/model/SManga;", "Leu/kanade/tachiyomi/source/model/SChapter;",
        "Leu/kanade/tachiyomi/source/model/MangasPage;", "Leu/kanade/tachiyomi/source/model/Filter;",
        "Leu/kanade/tachiyomi/source/model/FilterList;", "Leu/kanade/tachiyomi/source/model/Page;",
        "Leu/kanade/tachiyomi/extension/ExtensionGenerated;",
        "Lrx/Observable;", "Lrx/Observer;", "Lrx/Scheduler;", "Lrx/Schedulers;",
        "Lrx/subjects/PublishSubject;", "Lrx/subjects/BehaviorSubject;",
        "Lrx/operators/Operator;", "Lrx/functions/Func1;", "Lrx/functions/Func2;",
    ];

    let mut refs = ReferenceSet::default();
    for cls in java_builtins {
        refs.classes.insert(ExtClassRef { descriptor: cls.to_string(), kind: ClassRefKind::Superclass });
    }
    refs
}

fn compute_gap(extension: &ReferenceSet, provided: &ReferenceSet) -> ReferenceSet {
    let mut gap = ReferenceSet::default();

    for cls in &extension.classes {
        let is_provided = provided.classes.iter().any(|p| p.descriptor == cls.descriptor);
        if !is_provided {
            gap.classes.insert(cls.clone());
        }
    }
    for m in &extension.methods {
        let is_provided = provided.methods.iter().any(|p| p.class == m.class && p.name == m.name && p.signature == m.signature);
        if !is_provided {
            gap.methods.insert(m.clone());
        }
    }
    for f in &extension.fields {
        let is_provided = provided.fields.iter().any(|p| p.class == f.class && p.name == f.name && p.descriptor == f.descriptor);
        if !is_provided {
            gap.fields.insert(f.clone());
        }
    }
    gap
}

fn print_report(ext_path: &str, ext_refs: &ReferenceSet, gap: &ReferenceSet) {
    println!("\n=== Gap Report: {} ===", ext_path);
    println!("Total external refs: classes={}, methods={}, fields={}",
        ext_refs.classes.len(), ext_refs.methods.len(), ext_refs.fields.len());
    println!("GAP: classes={}, methods={}, fields={}",
        gap.classes.len(), gap.methods.len(), gap.fields.len());

    if !gap.classes.is_empty() {
        println!("\n--- Missing Classes ---");
        let mut by_kind: BTreeMap<ClassRefKind, Vec<&ExtClassRef>> = BTreeMap::new();
        for c in &gap.classes {
            by_kind.entry(c.kind.clone()).or_default().push(c);
        }
        for (kind, classes) in by_kind {
            println!("  {:?} ({})", kind, classes.len());
            for c in classes.iter().take(20) {
                println!("    {}", c.descriptor);
            }
            if classes.len() > 20 {
                println!("    ... and {} more", classes.len() - 20);
            }
        }
    }

    if !gap.methods.is_empty() {
        println!("\n--- Missing Methods ---");
        let mut by_class: BTreeMap<String, Vec<&ExtMethodRef>> = BTreeMap::new();
        for m in &gap.methods {
            by_class.entry(m.class.clone()).or_default().push(m);
        }
        for (class, methods) in by_class {
            println!("  {} ({})", class, methods.len());
            for m in methods.iter().take(15) {
                println!("    {} {} [{}]", m.name, m.signature, match m.invoke_kind {
                    InvokeKind::Virtual => "virtual", InvokeKind::Static => "static",
                    InvokeKind::Direct => "direct", InvokeKind::Super => "super",
                    InvokeKind::Interface => "interface",
                });
            }
            if methods.len() > 15 {
                println!("    ... and {} more", methods.len() - 15);
            }
        }
    }

    if !gap.fields.is_empty() {
        println!("\n--- Missing Fields ---");
        let mut by_class: BTreeMap<String, Vec<&ExtFieldRef>> = BTreeMap::new();
        for f in &gap.fields {
            by_class.entry(f.class.clone()).or_default().push(f);
        }
        for (class, fields) in by_class {
            println!("  {} ({})", class, fields.len());
            for f in fields.iter().take(15) {
                println!("    {} {} {} [{}]", f.name, f.descriptor, f.access_kind, if f.is_static { "static" } else { "instance" });
            }
            if fields.len() > 15 {
                println!("    ... and {} more", fields.len() - 15);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: gapanalyzer <path-to-extension.apk> [--json]");
        std::process::exit(1);
    }

    let json_output = args.iter().any(|a| a == "--json");
    let apk_paths: Vec<&str> = args.iter()
        .filter(|a| !a.starts_with("--"))
        .map(|s| s.as_str())
        .collect();

    let provided = provided_by_vm();

    for apk_path in apk_paths {
        let data = fs::read(apk_path).unwrap_or_else(|e| {
            eprintln!("Failed to read {}: {}", apk_path, e);
            std::process::exit(1);
        });

        let dex = DexFile::parse(&data).unwrap_or_else(|e| {
            eprintln!("Failed to parse {}: {}", apk_path, e);
            std::process::exit(1);
        });

        let ext_refs = analyze_dex(&dex);
        let gap = compute_gap(&ext_refs, &provided);

        if json_output {
            let report = serde_json::json!({
                "apk": apk_path,
                "total_refs": {
                    "classes": ext_refs.classes.len(),
                    "methods": ext_refs.methods.len(),
                    "fields": ext_refs.fields.len(),
                },
                "gap": {
                    "classes": gap.classes.len(),
                    "methods": gap.methods.len(),
                    "fields": gap.fields.len(),
                },
                "missing_classes": gap.classes.iter().map(|c| serde_json::json!({
                    "descriptor": c.descriptor,
                    "kind": format!("{:?}", c.kind)
                })).collect::<Vec<_>>(),
                "missing_methods": gap.methods.iter().map(|m| serde_json::json!({
                    "class": m.class,
                    "name": m.name,
                    "signature": m.signature,
                    "static": m.is_static,
                    "kind": format!("{:?}", m.invoke_kind)
                })).collect::<Vec<_>>(),
                "missing_fields": gap.fields.iter().map(|f| serde_json::json!({
                    "class": f.class,
                    "name": f.name,
                    "descriptor": f.descriptor,
                    "static": f.is_static,
                    "access": format!("{:?}", f.access_kind)
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        } else {
            print_report(apk_path, &ext_refs, &gap);
        }
    }
}