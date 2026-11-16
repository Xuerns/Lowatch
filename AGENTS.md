# AGENTS.md — Strict Guidelines & Working Principles for AI Agents (Rust)

This document is the **absolute working contract** for every AI Coding Agent working in this repository. Your primary goal is to write Rust code that is safe, idiomatic, measurable, and logically correct—**not merely code that compiles or makes the test indicators turn green through shortcuts.**

---

## 1. Test Integrity (Zero Tolerance for Test Manipulation)

Tests are the specification of the system's correctness. If a test fails (*FAIL*), it means the **implementation is wrong**, not the test.

### STRICTLY FORBIDDEN:
- **Changing Assertions:** Never change the expected values in `assert!`, `assert_eq!`, `assert_ne!`, or `matches!` just to match the output of code that is currently *buggy*.
- **Deleting or Disabling Tests:** Never delete a `#[test]` function, comment out test code, or add the `#[ignore]` / `#[should_panic]` attributes to hide failures.
- **Hardcoding / Fake Stubbing:** Never write logic like `if input == test_case { return expected_output }` in production code just to trick the test runner.
- **Weakening Validation in Tests:** Never replace a specific check (e.g. verifying a particular error variant) with a loose one (e.g. merely `assert!(result.is_err())`).

### WHAT YOU MUST DO:
- Trace the root cause in the business logic or data flow of the production code.
- You **may only** modify an existing test if the user explicitly requests a change to the business specification, and you must explain the reason for the test change to the user.

---

## 2. Ownership, Borrowing, and Lifetimes Discipline

The borrow checker in Rust exists to guarantee memory and concurrency safety. Do not fight it by damaging the code architecture.

### STRICTLY FORBIDDEN:
- **Spamming `.clone()`:** Do not blindly add `.clone()` or `.to_owned()` just to silence *borrow checker* errors (`cannot move out of...` or `borrowed value does not live long enough`) without understanding who should be the owner of that data.
- **Misusing Smart Pointers:** Do not immediately wrap types in `Rc<RefCell<T>>` or `Arc<Mutex<T>>` just because you struggle to structure *ownership* or *mutable references*, unless it is genuinely required for shared state across *threads* or for a legitimate graph structure.
- **Faking `'static` Lifetimes:** Do not force a `T: 'static` bound or use `Box::leak` just because you are too lazy to define the correct *lifetime annotations* (`'a`).

### WHAT YOU MUST DO:
- Design the data flow clearly: decide which functions need full ownership (`T`), which only need to read (`&T`), and which need to modify (`&mut T`).
- Use *slices* (`&[T]`, `&str`) instead of references to full collections (`&Vec<T>`, `&String`) in function parameters.
- When you run into a complex *borrow checker* problem, restructure the *scope* of your variables or split the struct into more modular parts.

---

## 3. Memory Safety & the Ban on `unsafe`

Rust's main strength is its compile-time guarantee of *memory safety*.

### STRICTLY FORBIDDEN:
- **Using `unsafe` Blocks:** You are strictly forbidden from writing `unsafe { ... }` blocks, `unsafe fn`, or `unsafe impl` to bypass compiler rules.
- **Type Transmutation:** Never use `std::mem::transmute`, *raw pointers* (`*const T`, `*mut T`), or `MaybeUninit` as a shortcut to work around type or *lifetime* problems.

### EXCEPTION:
- `unsafe` is only permitted when the user specifically requests it (for example, for FFI integration with C) and it must be accompanied by a `// SAFETY: ...` comment that explains in detail why the invariants hold.

---

## 4. Explicit Error Handling

Production code must not *crash* (`panic!`) because of invalid input or predictable I/O failures.

### STRICTLY FORBIDDEN:
- **Using `.unwrap()` and `.expect()` in Production Code:** Never use either outside of test code (`#[cfg(test)]`), unless you can prove mathematically/statically that the value can never be `None`/`Err` (and even then, *pattern matching* is preferred).
- **Silently Swallowing Errors:** Ignoring a `Result` with `let _ = risky_function();` or converting it with `.ok()` without handling the failure scenario is forbidden.
- **Using `panic!`, `todo!`, or `unimplemented!`:** Do not leave these macros in code that is claimed to be finished.

### WHAT YOU MUST DO:
- Use `Result<T, E>` and propagate errors with the `?` operator.
- Create meaningful error types (using a dedicated `enum`, `thiserror` for *libraries*, or `anyhow` for the *application layer*, following project conventions).
- Use idiomatic combinators such as `.map_err()`, `.ok_or_else()`, `if let`, or `let ... else`.

---

## 5. Compiler & Clippy Compliance

Warnings from `rustc` and `clippy` are signals of potential *bugs* or non-idiomatic code.

### STRICTLY FORBIDDEN:
- **Silencing Linters:** Never add attributes such as `#[allow(dead_code)]`, `#[allow(unused_variables)]`, `#[allow(clippy::all)]`, or similar just to make the *build* free of warnings.
- **Bypassing Type Checking with `Any`:** Do not use `Box<dyn Any>` or serialization to `serde_json::Value` solely to avoid writing the proper generic types or *traits*.

### WHAT YOU MUST DO:
- If there is `dead_code`, remove the unused code or connect it to the module that should be calling it.
- Run `cargo check`, `cargo test`, and `cargo clippy -- -D warnings` as the standard verification before declaring work complete.
- Make full use of Rust's type system (*Type-Driven Design*): use `enum`s and the `newtype pattern` so that invalid states cannot be represented in code (*make illegal states unrepresentable*).

---

## 6. Protocol When Stuck (Stuck Protocol)

If you fail to compile the code or resolve a *bug* after several attempts:
1. **STOP** making random changes (*shotgun debugging*) such as adding `.clone()`, changing `&` signs, or tinkering with tests.
2. **ANALYZE** the `rustc` error messages thoroughly—the Rust compiler gives very accurate structural hints.
3. **ASK THE USER** if there is ambiguity in the architectural design or domain logic, rather than forcing a poor technical compromise.