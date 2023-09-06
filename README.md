VPlugin is discontinued for several reasons and any source code using it is better off either forking the source code or switching to another solution.
Main reasons are:

- Lack of time, as I've been spending time on more useful and easier to manage projects, in addition to the two games I've been writing for some time.
- Unsafe, let's face it, VPlugin works entirely on unsafety. Also anyone could just hijack your process and there's not much you could do.
- Unsuited for Rust, the ownership model just doesn't play well with VPlugin.
- Better solutions exist. Lua worked for me better and I also wrote my own code execution sandbox for situations like this (No, it won't be public, it's intended for private use).
- Final: I (@tseli0s) the enforced 2FA policy that recently took place. I plan to move away from the platform in a few months, and so should you.

That's all, bye :)
