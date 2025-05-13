# `repo-birthday` 🎂

Tells when your repos are celebrating their birthdays — and their age.

Requires an environment variable `GITHUB_ACCESS_TOKEN` with a GitHub token with ` public_repo` scope.

```console
$ cargo run -- <your GitHub username>
```

For example, for my case, `cargo run -- cuducos`.

Or with the `--ical`  option you can create a `.ical` file to add to your calendar:

```console
$ cargo run -- <your GitHub username> --ical
```
