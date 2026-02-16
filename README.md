# HAIVE

<img width="1660" height="380" alt="header-light" src="https://github.com/user-attachments/assets/0e6331bf-ff3d-407b-9f3d-0efd9c4e9af1" />

## Obiettivi del progetto
Lo scopo del progetto consiste nella realizzazione di un **modello di intelligenza artificiale** in grado di sfidare un altro agente intelligente -*artificiale o umano*- nel gioco di [Hive](https://en.wikipedia.org/wiki/Hive_(game))

## Organizzazione della repository
In questa repository GitHub è possibile trovare due root directory: `haive-engine` e `haive-report`. 
In `haive-engine` si trova il codice sorgente del proggetto, scritto totalmente in Rust ed una piccola parte in Python per automatizzare alcuni processi, ecco i moduli:

- `battler`: Modulo che permette di far sfidare il modello (haive) contro un altro modello arbitrario che implementa un server UHP.
- `data_generation`: Modulo per la generazione dei dati, fa sfidare due agenti *random* (che giocano eseguendo mosse casuali) e salva l'intera partita ed i risultati, usato per il training della rete neurale.
- `dataset`: Directory con i file generati per training e validation.
- `engine`: Modulo per avviare il server UHP.
- `hive_library`: Modulo centrale del progetto, è presente tutta la logica di gioco e del modello. Sono presenti la rappresentazione della board di gioco e dei pezzi per Minimax e NNUE, la rappresentazione dei player, il codice per eseguire partite e per implementare il server ed il client UHP.
- `trainig`: Modulo per allenare la rete neurale.

## Installazione ed uso
Questa sezione ti guiderà per configurare ed eseguire il progetto.

- ### Prerequisiti
-   **Rust Toolchain**: L'ultima versione di Rust e Cargo installati sulla macchina.

-   ### Compilazione ed esecuzione dell'engine

1.  **Clona la repository**
    ```bash
    git clone https://github.com/andrea-sq/hAIve.git
    cd hAIve
    ```

2.  **Entra nel crate `hive-engine`**
    ```bash
    cd hive-engine
    ```

3.  **Compila il progetto**
    Esegui il build dell'intero progetto nella subdirectory `hive-engine` con tutte le librerire ed eseguibili associati.
    ```bash
    cargo build --release --bin engine
    ```

4.  **Esegui**
    Se la compilazione è andata a buon fine, `hive-engine/cargo/release` conterrà un target eseguibile:
    ```bash
    cd cargo/release
    ./engine
    ```
