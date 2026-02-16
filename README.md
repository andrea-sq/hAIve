# HAIVE

<img width="1660" height="380" alt="header-light" src="https://github.com/user-attachments/assets/0e6331bf-ff3d-407b-9f3d-0efd9c4e9af1" />

## Obiettivi del progetto
Lo scopo del progetto consiste nella realizzazione di un **modello di intelligenza artificiale** in grado di sfidare un altro agente intelligente -*artificiale o umano*- nel gioco di [Hive](https://en.wikipedia.org/wiki/Hive_(game))

## Organizzazione della repository
In questa repository GitHub è possibile trovare due root directory: `hive-engine` e `haive-report`. 
In `haive-engine` si trova il codice sorgente del proggetto, scritto totalmente in Rust ed una piccola parte in Python per automatizzare alcuni processi, ecco i moduli:

- `battler`: Modulo che permette di far sfidare il modello (haive) contro un altro modello arbitrario che implementa un server UHP.
- `data_generation`: Modulo per la generazione dei dati, fa sfidare due agenti *random* (che giocano eseguendo mosse casuali) e salva l'intera partita ed i risultati, usato per il training della rete neurale.
- `dataset`: Directory con i file generati per training e validation.
- `engine`: Modulo per avviare il server UHP.
- `hive_library`: Modulo centrale del progetto, è presente tutta la logica di gioco e del modello. Sono presenti la rappresentazione della board di gioco e dei pezzi per Minimax e NNUE, la rappresentazione dei player, il codice per eseguire partite e per implementare il server ed il client UHP.
- `trainig`: Modulo per allenare la rete neurale.

## Installazione ed uso
Questa sezione ti guiderà per configurare, compilare ed eseguire i vari moduli del progetto.

### Prerequisiti
-   **Rust Toolchain**: L'ultima versione di Rust e Cargo installati sulla macchina.
-  **Clona la repository**
    ```bash
    git clone https://github.com/andrea-sq/hAIve.git
    cd hAIve
    ```
### Compilazione ed esecuzione dell'engine

1.  **Entra nel crate `hive-engine`**
    ```bash
    cd hive-engine
    ```

2.  **Compila il progetto**
    Esegui il build dell'engine, il core del progetto, nella subdirectory `hive-engine` con tutte le librerire ed eseguibili associati.
    ```bash
    cargo build --release --bin engine
    ```

3.  **Esegui**
    Se la compilazione è andata a buon fine, `hive-engine/cargo/release` conterrà il target eseguibile `engine`:
    ```bash
    cd cargo/release
    ./engine
    ```

### Compilazione ed esecuzione del tool di confronto
1.  **Entra nel crate `hive-engine`**
    ```bash
    cd hive-engine
    ```

2.  **Compila il progetto**
    Esegui il build del modulo `battler` nella subdirectory `hive-engine` con tutte le librerire ed eseguibili associati.
    ```bash
    cargo build --release --battler battler
    ```

3.  **Esegui**
    Se la compilazione è andata a buon fine, `hive-engine/cargo/release` conterrà il target eseguibile `battler`:
    ```bash
    cd cargo/release
    ./battler
    ```

### Compilazione ed esecuzione del tool di generazione dati
1.  **Entra nel crate `hive-engine`**
    ```bash
    cd hive-engine
    ```

2.  **Compila il progetto**
    Esegui il build del modulo `data_generation` nella subdirectory `hive-engine` con tutte le librerire ed eseguibili associati.
    ```bash
    cargo build --release --data-generation data-generation
    ```

3.  **Esegui**
    Se la compilazione è andata a buon fine, `hive-engine/cargo/release` conterrà il target eseguibile `data-generation`:
    ```bash
    cd cargo/release
    ./data-generation
    ```
