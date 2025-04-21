### Projet : Essaim de Robots pour l'Exploration et l'Étude Astrobiologique (EREEA)

# NEBULINK
![image](https://github.com/user-attachments/assets/468ef0f8-15d8-41f3-b010-711738c24782)


## Group members:

- Gregoire Lequippe
- Ali Loudagh
- Nahel Kini
- Roman Sabechkine

This Rust project demonstrates two different UI approaches for building a robot simulation:

1. **Terminal-based UI with Ratatui** (default, in `main` branch)
2. **Graphical UI with Bevy** (in `bevy-version` branch)

---

## 📦 Project Structure

- `main` branch: Implements a terminal-based UI using [Ratatui](https://github.com/ratatui-org/ratatui).
- `bevy-version` branch: Uses [Bevy](https://bevyengine.org/) for a real-time graphical interface.

---

## 🚀 Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/Gregoire111278/RustProject.git
cd RustProject
```

### 2. Build and run the Ratatui version (default)

```bash
cargo run
```

### 3. Switch to the Bevy version

```bash
git checkout bevy-version
cargo run
```

### 4. Run Tests (currently only in Ratatui version i.e. main branch)

You can run the test suite using:

```bash
cargo test
```

---

## 📂 Final Branches

- `main`: Terminal-based simulation (Ratatui)
- `bevy-version`: Graphical simulation (Bevy)

---

More details in:
https://github.com/Gregoire111278/RustProject/wiki
