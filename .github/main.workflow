workflow "New workflow" {
  on = "push"
  resolves = ["Rust GitHub Action"]
}

action "Rust GitHub Action" {
  uses = "icepuma/rust-action@master"
  args = "cargo build"
}
