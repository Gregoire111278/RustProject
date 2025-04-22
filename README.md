### Projet : Essaim de Robots pour l'Exploration et l'Étude Astrobiologique (EREEA)

# NEBULINK
<img src="https://github.com/user-attachments/assets/468ef0f8-15d8-41f3-b010-711738c24782" alt="NEBULINK Logo" width="150"/>

NEBULINK is a simulation platform for a swarm of autonomous robots designed to explore extraterrestrial terrain. These robots cooperate to analyze geology, collect samples, and identify areas of scientific interest, while sharing their data upon their return to the station. The project emphasizes performance, concurrent programming, and collective intelligence.

## Made by:

- Gregoire Lequippe
- Ali Loudagh
- Nahel Kini
- Roman Sabechkine

<p align="center">
  <img src="https://github.com/user-attachments/assets/1fd8a53f-a4b7-4eca-af63-29b1cbc14b05" alt="Swarm Robots on Alien Planet" width="600"/>
</p>

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
- `nahel`: Graphical simulation (Bevy)

---

More details in:
https://github.com/Gregoire111278/RustProject/wiki
