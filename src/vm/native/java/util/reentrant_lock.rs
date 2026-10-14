//! java.util.concurrent.locks.ReentrantLock / Condition host shims.

use crate::vm::native::*;

pub(crate) fn reentrant_lock_init(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::ReentrantLock { locked }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    // Fairness controls waiter ordering; it must not make a new lock held.
    // This VM executes synchronously, so both constructors start unlocked.
    *locked = false;
    Ok(JValue::Null)
}

pub(crate) fn reentrant_lock_lock(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::ReentrantLock { locked }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    *locked = true;
    Ok(JValue::Null)
}

pub(crate) fn reentrant_lock_unlock(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::ReentrantLock { locked }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    *locked = false;
    Ok(JValue::Null)
}

pub(crate) fn reentrant_lock_new_condition(vm: &mut Vm, _args: &[JValue]) -> R {
    alloc(vm, "Ljava/util/concurrent/locks/Condition;", Native::Opaque)
}

pub(crate) fn condition_await_nanos(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Long(0))
}

pub(crate) fn condition_await(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Null)
}

pub(crate) fn condition_signal(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Null)
}

pub(crate) fn condition_signal_all(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Null)
}

// java.util.concurrent.locks.ReentrantReadWriteLock host shims.
//
// This VM runs one Java thread, so a read/write lock never contends: a view is
// always free to take and no waiter can queue. `readLock()`/`writeLock()` hand
// out a fresh view per call because bytecode holds them as separate references
// (`lock.readLock().lock()` builds one, `unlock()` builds another).

pub(crate) fn rw_lock_view(vm: &mut Vm, desc: &'static str) -> R {
    alloc(vm, desc, Native::Opaque)
}

pub(crate) fn rw_read_lock(vm: &mut Vm, _args: &[JValue]) -> R {
    rw_lock_view(
        vm,
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
    )
}

pub(crate) fn rw_write_lock(vm: &mut Vm, _args: &[JValue]) -> R {
    rw_lock_view(
        vm,
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
    )
}

pub(crate) fn rw_lock_void(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Null)
}

pub(crate) fn rw_try_lock(_vm: &mut Vm, _args: &[JValue]) -> R {
    // Nothing else can hold it here, so a non-blocking acquire always wins.
    Ok(JValue::Int(1))
}

pub(crate) fn rw_hold_zero(_vm: &mut Vm, _args: &[JValue]) -> R {
    // A view is handed out per call and never queues a waiter, so no thread
    // holds a read or write count in this VM.
    Ok(JValue::Int(0))
}

/// Native methods for Ljava/util/concurrent/locks/ReentrantLock;
pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/util/concurrent/locks/ReentrantLock;",
        "<init>",
        "()V",
        true,
        reentrant_lock_init
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantLock;",
        "<init>",
        "(Z)V",
        true,
        reentrant_lock_init
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantLock;",
        "lock",
        "()V",
        true,
        reentrant_lock_lock
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantLock;",
        "unlock",
        "()V",
        true,
        reentrant_lock_unlock
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantLock;",
        "newCondition",
        "()Ljava/util/concurrent/locks/Condition;",
        true,
        reentrant_lock_new_condition
    ),
    // ReentrantReadWriteLock and its read/write views.
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "<init>",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "<init>",
        "(Z)V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "readLock",
        "()Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
        true,
        rw_read_lock
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "writeLock",
        "()Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
        true,
        rw_write_lock
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "lock",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "lockInterruptibly",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "unlock",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "tryLock",
        "()Z",
        true,
        rw_try_lock
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "tryLock",
        "(JLjava/util/concurrent/TimeUnit;)Z",
        true,
        rw_try_lock
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "getReadHoldCount",
        "()I",
        true,
        rw_hold_zero
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "getWriteHoldCount",
        "()I",
        true,
        rw_hold_zero
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock;",
        "newCondition",
        "()Ljava/util/concurrent/locks/Condition;",
        true,
        reentrant_lock_new_condition
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
        "<init>",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
        "lock",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
        "lockInterruptibly",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
        "unlock",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
        "tryLock",
        "()Z",
        true,
        rw_try_lock
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;",
        "newCondition",
        "()Ljava/util/concurrent/locks/Condition;",
        true,
        reentrant_lock_new_condition
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
        "<init>",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
        "lock",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
        "lockInterruptibly",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
        "unlock",
        "()V",
        true,
        rw_lock_void
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
        "tryLock",
        "()Z",
        true,
        rw_try_lock
    ),
    ne!(
        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;",
        "newCondition",
        "()Ljava/util/concurrent/locks/Condition;",
        true,
        reentrant_lock_new_condition
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, SandboxOptions};

    #[test]
    fn both_constructors_start_unlocked() {
        let data = std::fs::read("fixtures/classes.dex").unwrap();
        let mut ctx = Context::new_with(&data, SandboxOptions::allow_all()).unwrap();
        let vm = ctx.vm();
        let lock = alloc(
            vm,
            "Ljava/util/concurrent/locks/ReentrantLock;",
            Native::ReentrantLock { locked: true },
        )
        .unwrap();
        reentrant_lock_init(vm, &[lock]).unwrap();
        assert!(matches!(
            payload(vm, lock),
            Some(Native::ReentrantLock { locked: false })
        ));
        reentrant_lock_init(vm, &[lock, JValue::Int(1)]).unwrap();
        assert!(matches!(
            payload(vm, lock),
            Some(Native::ReentrantLock { locked: false })
        ));
    }
}
