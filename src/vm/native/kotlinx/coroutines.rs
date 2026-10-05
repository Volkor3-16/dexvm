//! kotlinx.coroutines host shims.

use crate::vm::native::*;

pub(crate) fn coroutine_scope_create(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)
}
pub(crate) fn coroutines_global_scope(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(vm, "Lkotlinx/coroutines/GlobalScope;", Native::Opaque)
}
pub(crate) fn coroutines_dispatchers_io(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(
        vm,
        "Lkotlinx/coroutines/CoroutineDispatcher;",
        Native::Opaque,
    )
}
pub(crate) fn coroutines_dispatchers_main(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(
        vm,
        "Lkotlinx/coroutines/MainCoroutineDispatcher;",
        Native::Opaque,
    )
}
pub(crate) fn coroutines_run_blocking(vm: &mut Vm, args: &[JValue]) -> R {
    inv_virt(
        vm,
        args[1],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[JValue::Null, JValue::Null],
    )
}
pub(crate) fn coroutines_with_timeout(vm: &mut Vm, args: &[JValue]) -> R {
    inv_virt(
        vm,
        args[1],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[JValue::Null, args[2]],
    )
}
pub(crate) fn completable_deferred_default(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(
        vm,
        "Lkotlinx/coroutines/CompletableDeferred;",
        Native::Deferred {
            value: JValue::Null,
            error: JValue::Null,
        },
    )
}
pub(crate) fn mutex_default(vm: &mut Vm, _args: &[JValue]) -> R {
    // Create a simple mutex implementation using Native::Mutex
    // The class will be set to the Mutex interface, which now has default methods
    let mutex_class = vm.ensure_class_by_desc("Lkotlinx/coroutines/sync/Mutex;").map_err(|e| iae(vm, &e.to_string()))?;
    let mutex_obj = vm.arena.alloc(
        mutex_class,
        Vec::new(),
        Some(Native::Mutex { locked: false }),
    );
    Ok(JValue::Obj(mutex_obj))
}
pub(crate) fn mutex_lock(vm: &mut Vm, args: &[JValue]) -> R {
    // Non-suspend lock (not commonly used)
    let Some(Native::Mutex { locked }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    *locked = true;
    Ok(JValue::Null)
}

pub(crate) fn mutex_try_lock(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::Mutex { locked }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    if *locked {
        return Ok(JValue::Int(0));
    }
    *locked = true;
    Ok(JValue::Int(1))
}

pub(crate) fn mutex_unlock(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::Mutex { locked }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    if !*locked {
        return Err(iae(vm, "Mutex is not locked"));
    }
    *locked = false;
    Ok(JValue::Null)
}

pub(crate) fn mutex_is_locked(vm: &mut Vm, args: &[JValue]) -> R {
    match payload(vm, args[0]) {
        Some(Native::Mutex { locked }) => Ok(JValue::Int(i32::from(*locked))),
        _ => Err(npe(vm)),
    }
}

/// Suspend lock function for Mutex interface: `lock(block: suspend () -> T): T`
/// This is called as an extension function on Mutex with a continuation.
pub(crate) fn mutex_suspend_lock(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = Mutex receiver
    // args[1] = block: suspend () -> T
    // args[2] = continuation
    let mutex = match args.get(0) {
        Some(JValue::Obj(o)) => *o,
        _ => return Err(npe(vm)),
    };
    let block = match args.get(1) {
        Some(v) => *v,
        _ => return Err(npe(vm)),
    };
    let cont = match args.get(2) {
        Some(v) => *v,
        _ => return Err(npe(vm)),
    };
    // Acquire the lock
    let was_locked = {
        let Some(Native::Mutex { locked }) = payload_mut(vm, JValue::Obj(mutex)) else {
            return Err(npe(vm));
        };
        let was_locked = *locked;
        if was_locked {
            // Lock is already held - in a real implementation this would suspend
            // For our synchronous VM, we'll just execute and hope for the best
        }
        *locked = true;
        was_locked
    };
    
    // Execute the block synchronously - release the mutable borrow first
    let result = vm.invoke_virtual_args(block, "invoke", "()Ljava/lang/Object;", vec![]);
    
    // Release the lock
    {
        let Some(Native::Mutex { locked }) = payload_mut(vm, JValue::Obj(mutex)) else {
            return Err(npe(vm));
        };
        *locked = false;
    }
    
    // Resume the continuation with the result
    let result_val = match result {
        Ok(res) => res,
        Err(_e) => JValue::Obj(vm.err_npe()),
    };
    let _ = vm.invoke_virtual_args(cont, "resumeWith", "(Ljava/lang/Object;)V", vec![result_val]);
    Ok(JValue::Null)
}

/// Static extension function MutexKt.lock
pub(crate) fn mutex_kt_lock(vm: &mut Vm, args: &[JValue]) -> R {
    // Same as mutex_suspend_lock but as a static function
    mutex_suspend_lock(vm, args)
}

pub(crate) fn coroutines_launch_default(vm: &mut Vm, args: &[JValue]) -> R {
    let _ = vm.invoke_virtual_args(
        args[3],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        vec![args[0], JValue::Null],
    );
    alloc(vm, "Lkotlinx/coroutines/Job;", Native::Opaque)
}
/// `delay(millis, continuation)`: this VM runs coroutines synchronously, so
/// there's nothing to actually suspend for — just resume immediately.
pub(crate) fn coroutines_delay(vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(lazy_unit_instance(vm))
}
pub(crate) fn coroutines_supervisor_job_default(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(vm, "Lkotlinx/coroutines/CompletableJob;", Native::Opaque)
}
pub(crate) fn coroutines_job_cancel_default(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Null)
}
pub(crate) fn coroutines_is_active(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Int(1))
}
pub(crate) fn coroutines_get_immediate(_vm: &mut Vm, args: &[JValue]) -> R {
    Ok(args[0])
}
pub(crate) fn coroutines_supervisor_scope(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(
        vm,
        args[0],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[scope, args[1]],
    )
}
pub(crate) fn coroutines_scope(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(
        vm,
        args[0],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[scope, args[1]],
    )
}
pub(crate) fn coroutines_with_context(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(
        vm,
        args[1],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[scope, args[2]],
    )
}
pub(crate) fn coroutines_async_default(vm: &mut Vm, args: &[JValue]) -> R {
    let value = inv_virt(
        vm,
        args[3],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[args[0], JValue::Null],
    )
    .unwrap_or(JValue::Null);
    alloc(
        vm,
        "Lkotlinx/coroutines/Deferred;",
        Native::Deferred {
            value,
            error: JValue::Null,
        },
    )
}
pub(crate) fn deferred_await(vm: &mut Vm, args: &[JValue]) -> R {
    match payload(vm, args[0]) {
        Some(Native::Deferred { value, .. }) => Ok(*value),
        _ => Err(npe(vm)),
    }
}
pub(crate) fn deferred_await_all(vm: &mut Vm, args: &[JValue]) -> R {
    let values = coll_elems(vm, args[0])?
        .into_iter()
        .map(|v| deferred_await(vm, &[v]))
        .collect::<Result<Vec<_>, _>>()?;
    list_alloc(vm, values)
}

// Deferred instance method - await (instance method on Deferred)
pub(crate) fn coroutines_deferred_await_instance(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = Deferred receiver
    match payload(vm, args[0]) {
        Some(Native::Deferred { value, .. }) => Ok(*value),
        _ => Err(npe(vm)),
    }
}

// CoroutineScope instance methods
pub(crate) fn coroutines_launch(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = CoroutineScope receiver
    // args[1] = block: suspend () -> T
    let block = args[1];
    let _ = vm.invoke_virtual_args(block, "invoke", "()Ljava/lang/Object;", vec![]);
    alloc(vm, "Lkotlinx/coroutines/Job;", Native::Opaque)
}

pub(crate) fn coroutines_async(vm: &mut Vm, args: &[JValue]) -> R {
    let block = args[1];
    let value = vm.invoke_virtual_args(block, "invoke", "()Ljava/lang/Object;", vec![]).unwrap_or(JValue::Null);
    alloc(
        vm,
        "Lkotlinx/coroutines/Deferred;",
        Native::Deferred {
            value,
            error: JValue::Null,
        },
    )
}

// CoroutineScope instance methods
pub(crate) fn coroutines_coroutine_scope_instance(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(
        vm,
        args[0],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[scope, args[1]],
    )
}

pub(crate) fn coroutines_supervisor_scope_instance(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(
        vm,
        args[0],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[scope, args[1]],
    )
}

pub(crate) fn coroutines_with_context_instance(vm: &mut Vm, args: &[JValue]) -> R {
    let scope = alloc(vm, "Lkotlinx/coroutines/CoroutineScope;", Native::Opaque)?;
    inv_virt(
        vm,
        args[1],
        "invoke",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[scope, args[2]],
    )
}



// Job instance methods
pub(crate) fn coroutines_job_join(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = Job receiver
    // For our synchronous VM, job is already complete
    Ok(JValue::Null)
}

pub(crate) fn coroutines_job_is_cancelled(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = Job receiver
    Ok(JValue::Int(0))
}

pub(crate) fn coroutines_job_is_completed(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = Job receiver
    Ok(JValue::Int(1))
}

pub(crate) fn coroutines_job_cancel(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = Job receiver
    Ok(JValue::Null)
}

pub(crate) fn coroutines_job_children(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = Job receiver
    list_alloc(vm, vec![])
}

// Deferred instance methods
pub(crate) fn coroutines_deferred_await(vm: &mut Vm, args: &[JValue]) -> R {
    match payload(vm, args[0]) {
        Some(Native::Deferred { value, .. }) => Ok(*value),
        _ => Err(npe(vm)),
    }
}

// Job instance methods on CoroutineScope
pub(crate) fn coroutines_job_join_instance(vm: &mut Vm, args: &[JValue]) -> R {
    // args[0] = Job receiver
    Ok(JValue::Null)
}

pub(crate) fn coroutines_job_is_cancelled_instance(vm: &mut Vm, args: &[JValue]) -> R {
    Ok(JValue::Int(0))
}

pub(crate) fn coroutines_job_is_completed_instance(vm: &mut Vm, args: &[JValue]) -> R {
    Ok(JValue::Int(1))
}

pub(crate) fn coroutines_job_cancel_instance(vm: &mut Vm, args: &[JValue]) -> R {
    Ok(JValue::Null)
}

pub(crate) fn coroutines_job_children_instance(vm: &mut Vm, args: &[JValue]) -> R {
    list_alloc(vm, vec![])
}


pub(crate) const TABLE: &[NativeEntry] = &[
    ne!("Lkotlinx/coroutines/CoroutineScopeKt;", "CoroutineScope", "(Lkotlin/coroutines/CoroutineContext;)Lkotlinx/coroutines/CoroutineScope;", false, coroutine_scope_create),
    ne!("Lkotlinx/coroutines/GlobalScope;", "getInstance", "()Lkotlinx/coroutines/GlobalScope;", false, coroutines_global_scope),
    ne!("Lkotlinx/coroutines/Dispatchers;", "getIO", "()Lkotlinx/coroutines/CoroutineDispatcher;", false, coroutines_dispatchers_io),
    ne!("Lkotlinx/coroutines/Dispatchers;", "getMain", "()Lkotlinx/coroutines/MainCoroutineDispatcher;", false, coroutines_dispatchers_main),
    ne!("Lkotlinx/coroutines/BuildersKt;", "runBlockingK", "(Lkotlin/coroutines/CoroutineContext;Lkotlin/jvm/functions/Function2;)Ljava/lang/Object;", false, coroutines_run_blocking),
    ne!("Lkotlinx/coroutines/BuildersKt;", "runBlockingK$default", "(Lkotlin/coroutines/CoroutineContext;Lkotlin/jvm/functions/Function2;ILjava/lang/Object;)Ljava/lang/Object;", false, coroutines_run_blocking),
    ne!("Lkotlinx/coroutines/TimeoutKt;", "withTimeout-KLykuaI", "(JLkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, coroutines_with_timeout),
    ne!("Lkotlinx/coroutines/CompletableDeferredKt;", "CompletableDeferred$default", "(Lkotlinx/coroutines/Job;ILjava/lang/Object;)Lkotlinx/coroutines/CompletableDeferred;", false, completable_deferred_default),
    ne!("Lkotlinx/coroutines/BuildersKt;", "launch$default", "(Lkotlinx/coroutines/CoroutineScope;Lkotlin/coroutines/CoroutineContext;Lkotlinx/coroutines/CoroutineStart;Lkotlin/jvm/functions/Function2;ILjava/lang/Object;)Lkotlinx/coroutines/Job;", false, coroutines_launch_default),
    ne!("Lkotlinx/coroutines/BuildersKt;", "async$default", "(Lkotlinx/coroutines/CoroutineScope;Lkotlin/coroutines/CoroutineContext;Lkotlinx/coroutines/CoroutineStart;Lkotlin/jvm/functions/Function2;ILjava/lang/Object;)Lkotlinx/coroutines/Deferred;", false, coroutines_async_default),
    ne!("Lkotlinx/coroutines/CoroutineScopeKt;", "coroutineScope", "(Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, coroutines_scope),
    ne!("Lkotlinx/coroutines/BuildersKt;", "withContext", "(Lkotlin/coroutines/CoroutineContext;Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, coroutines_with_context),
    ne!("Lkotlinx/coroutines/Deferred;", "await", "(Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, deferred_await),
    ne!("Lkotlinx/coroutines/AwaitKt;", "awaitAll", "(Ljava/util/Collection;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, deferred_await_all),
    ne!("Lkotlinx/coroutines/sync/MutexKt;", "Mutex$default", "(ZILjava/lang/Object;)Lkotlinx/coroutines/sync/Mutex;", false, mutex_default),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "lock", "()V", true, mutex_lock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "lock", "(Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, mutex_suspend_lock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "tryLock", "()Z", true, mutex_try_lock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "unlock", "()V", true, mutex_unlock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "isLocked", "()Z", true, mutex_is_locked),
    ne!("Lkotlinx/coroutines/sync/MutexKt;", "lock", "(Lkotlinx/coroutines/sync/Mutex;Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, mutex_kt_lock),
    ne!("Lkotlinx/coroutines/SupervisorKt;", "SupervisorJob$default", "(Lkotlinx/coroutines/Job;ILjava/lang/Object;)Lkotlinx/coroutines/CompletableJob;", false, coroutines_supervisor_job_default),
    ne!("Lkotlinx/coroutines/SupervisorKt;", "supervisorScope", "(Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, coroutines_supervisor_scope),
    ne!("Lkotlinx/coroutines/Job;", "cancel$default", "(Lkotlinx/coroutines/Job;Ljava/util/concurrent/CancellationException;ILjava/lang/Object;)V", false, coroutines_job_cancel_default),
    ne!("Lkotlinx/coroutines/CoroutineScopeKt;", "cancel$default", "(Lkotlinx/coroutines/CoroutineScope;Ljava/util/concurrent/CancellationException;ILjava/lang/Object;)V", false, coroutines_job_cancel_default),
    ne!("Lkotlinx/coroutines/CoroutineScopeKt;", "cancel$default", "(Lkotlinx/coroutines/CoroutineScope;Ljava/lang/Throwable;ILjava/lang/Object;)V", false, coroutines_job_cancel_default),
    ne!("Lkotlinx/coroutines/CoroutineScopeKt;", "isActive", "(Lkotlinx/coroutines/CoroutineScope;)Z", false, coroutines_is_active),
    ne!("Lkotlinx/coroutines/MainCoroutineDispatcher;", "getImmediate", "()Lkotlinx/coroutines/MainCoroutineDispatcher;", true, coroutines_get_immediate),
    ne!("Lkotlinx/coroutines/DelayKt;", "delay", "(JLkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, coroutines_delay),
    ne!("Lkotlinx/coroutines/DelayKt;", "delay-VtjQ1oo", "(JLkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, coroutines_delay),

    // CoroutineScope instance methods
    ne!("Lkotlinx/coroutines/CoroutineScope;", "launch", "(Lkotlin/jvm/functions/Function2;)Lkotlinx/coroutines/Job;", true, coroutines_launch),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "async", "(Lkotlin/jvm/functions/Function2;)Lkotlinx/coroutines/Deferred;", true, coroutines_async),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "coroutineScope", "(Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_coroutine_scope_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "supervisorScope", "(Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_supervisor_scope_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "withContext", "(Lkotlin/coroutines/CoroutineContext;Lkotlin/jvm/functions/Function2;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_with_context_instance),

    // Job instance methods
    ne!("Lkotlinx/coroutines/Job;", "join", "(Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_job_join),
    ne!("Lkotlinx/coroutines/Job;", "cancel", "()V", true, coroutines_job_cancel),
    ne!("Lkotlinx/coroutines/Job;", "children", "()Ljava/util/List;", true, coroutines_job_children),
    ne!("Lkotlinx/coroutines/Job;", "isCancelled", "()Z", true, coroutines_job_is_cancelled),
    ne!("Lkotlinx/coroutines/Job;", "isCompleted", "()Z", true, coroutines_job_is_completed),

    // Job instance methods on CoroutineScope
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobJoin", "(Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_job_join_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobIsCancelled", "()Z", true, coroutines_job_is_cancelled_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobIsCompleted", "()Z", true, coroutines_job_is_completed_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobCancel", "()V", true, coroutines_job_cancel_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobChildren", "()Ljava/util/List;", true, coroutines_job_children_instance),

    // Deferred instance methods
    ne!("Lkotlinx/coroutines/Deferred;", "await", "(Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_deferred_await),
    ne!("Lkotlinx/coroutines/Deferred;", "await", "(Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_deferred_await_instance),

    // Job instance methods on CoroutineScope
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobJoin", "(Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_job_join_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobIsCancelled", "()Z", true, coroutines_job_is_cancelled_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobIsCompleted", "()Z", true, coroutines_job_is_completed_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobCancel", "()V", true, coroutines_job_cancel_instance),
    ne!("Lkotlinx/coroutines/CoroutineScope;", "jobChildren", "()Ljava/util/List;", true, coroutines_job_children_instance),

    // Deferred instance methods
    ne!("Lkotlinx/coroutines/Deferred;", "await", "(Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, coroutines_deferred_await_instance),
];
