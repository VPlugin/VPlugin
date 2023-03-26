# Security Policy
This is the security policy for VPlugin. It's important that you report any vulnerability using the described methods below to ensure the safety of all projects using VPlugin. Keep in mind that managing your process's permission is not VPlugin's job. In other words, if you don't want to have the plugins access the host system for whatever reason, that's up to you.

## Reporting
There are two ways to report an issue:
- By privately reporting it,
- Or by [creating an issue](https://github.com/VPlugin/VPlugin/issues/).

The latter is recommended as it will let everyone know of their vulnerability, but it will also make it possible for other developers to work on this issue.
When creating the issue, use the ```Vulnerability Report``` template and fill each section with the requested data.

From there on, you should probably only keep up with the issue to watch updates on its progress. Optionally, you can contribute code to fix it faster.

## My vulnerability is marked as `wontfix`!
Although this is rare to happen, here are a few reasons why this may happen:
- The vulnerability is not caused by VPlugin itself, but some other dependency.
- Your version of VPlugin is outdated and a patch has already been done.

If you still get marked with `wontfix` despite not meeting any of the reasons, then you may want to contact the assigned people.

## Supported Versions
These versions are guaranteed to receive security updates (Through patch releases):

| Version | Supported          |
| ------- | ------------------ |
| 0.3.0   | :white_check_mark: |
| 0.2.1   | :white_check_mark: |
| 0.2.0   | :white_check_mark: |
| 0.1.0   | :x:                |
