with open('/home/volkor/git/dexvm/src/vm/native/kotlinx/coroutines.rs', 'r') as f:
    content = f.read()

# Add the static lock function to MutexKt
old_block = '''    ne!("Lkotlinx/coroutines/sync/Mutex;", "lock", "()V", true, mutex_lock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "lock", "(Ljava/lang/Object;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, mutex_lock_suspend),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "tryLock", "()Z", true, mutex_try_lock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "unlock", "()V", true, mutex_unlock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "isLocked", "()Z", true, mutex_is_locked),'''

new_block = '''    ne!("Lkotlinx/coroutines/sync/Mutex;", "lock", "()V", true, mutex_lock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "lock", "(Ljava/lang/Object;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", true, mutex_lock_suspend),
    ne!("Lkotlinx/coroutines/sync/MutexKt;", "lock", "(Lkotlinx/coroutines/sync/Mutex;Lkotlin/coroutines/Continuation;)Ljava/lang/Object;", false, mutex_kt_lock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "tryLock", "()Z", true, mutex_try_lock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "unlock", "()V", true, mutex_unlock),
    ne!("Lkotlinx/coroutines/sync/Mutex;", "isLocked", "()Z", true, mutex_is_locked),'''

content = content.replace(old_block, new_block)

with open('/home/volkor/git/dexvm/src/vm/native/kotlinx/coroutines.rs', 'w') as f:
    f.write(content)

print("Done")