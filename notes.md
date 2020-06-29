1. implemented a recursive crawler that spawns other instances of itself to crawl a dir
1. that was slow, so I copied ripgrep's approach by spawning a fixed num of tasks (akin to threads), which worked better but still slow
1. tried locking outside the push-loop (mention ripgrep's recent switch to a locked stack) and that was far worse, idk why
1. tried copy/pasting code as exactly the same but using std::thread instead of async_std::task, much much better
1. next step is to try buffering output, since I observed that elminating the printing puts us on par with find
1. amazingly, running a single threaded crawler is far more performant than multi-threaded, if the task is only to print the directory!
    1. what if we don't print anything? still the case? or is locking stdout the expensive part?