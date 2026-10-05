import subprocess
from concurrent.futures import ThreadPoolExecutor

battle_path = "./target/release/battler"


def run_battle(
    agent1_path,
    agent2_path,
    game_type="Base+MLP",
    timeout="1s",
    verbose=False,
    strategy1="iterative",
    strategy2="iterative",
):
    """
    Run a single battle between two agents with specified parameters.

    Args:
        agent1_path (str): Path to the first agent's executable.
        agent2_path (str): Path to the second agent's executable.
        game_type (str): Game type (default: 'hive').
        depth (int): Search depth for the AI (default: 3).
        timeout (int): Timeout per move in seconds (default: 30).
        verbose (bool): Enable verbose output (default: False).

    Returns:
        tuple: (winner, game_outcome, error_message) or (None, None, error_message) if an error occurs.
    """
    cmd = [
        battle_path,
        agent1_path,
        agent2_path,
        "--game-type",
        game_type,
        "--timeout",
        str(timeout),
        "--player1-name",
        "Agent1",
        "--player2-name",
        "Agent2",
        "--verbose" if verbose else "",
        "--player1-strategy",
        strategy1,
        "--player2-strategy",
        strategy2,
        "--player1-num-threads",
        "1",
        "--player2-num-threads",
        "1",
    ]
    """
        "--player1-table_mb",
        "2048",
        "--player2-table_mb",
        "2048",
        "--player1-double-step",
        "--player2-double-step",
        "--player1-null-move-pruning",
        "--player2-null-move-pruning",
        "--player1-quiet-search",
        "--player2-quiet-search",
        "--player1-background-ponder",
        "--player2-background-ponder",
    """
    try:
        print("Started match")
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=3600,  # 1 hour timeout for the entire match
        )
        if result.returncode != 0:
            return None, None, f"Battle program exited with error: {result.stderr}"
        print(result.stdout)

        # Parse the output for game result
        game_string = None
        outcome = None
        for line in result.stdout.split("\n"):
            if line.startswith("Game string:"):
                game_string = line.split(":", 1)[1].strip()
            elif line.startswith("Game over:"):
                outcome = line.split(":", 1)[1].strip()

        if not game_string or not outcome:
            return None, None, "Incomplete game information"

        return (
            outcome.split()[0],
            game_string,
            None,
        )  # Return winner (e.g., 'Agent1') and game string

    except subprocess.TimeoutExpired:
        return None, None, "Match timed out"
    except Exception as e:
        return None, None, str(e)


def main():
    agent1_path = "ai"  # Replace with actual path
    agent2_path = "/home/andrea/Packages/nokamute/target/release/nokamute"  # Replace with actual path
    num_matches = 100
    game_type = "Base+MLP"

    results = []

    with ThreadPoolExecutor(max_workers=28) as executor:
        futures = [
            executor.submit(
                run_battle,
                agent1_path,
                agent2_path,
                game_type,
                verbose=True,
                strategy1="iterative",
                strategy2="iterative",
            )
            for _ in range(num_matches)
        ]

        for future in concurrent.futures.as_completed(futures):
            winner, game_string, error = future.result()
            if error:
                results.append(("Error", error))
            else:
                results.append((winner, game_string))

    # Count results
    agent1_wins = sum(
        1 for w, _ in results if ("agent1" in w.lower() or "white" in w.lower())
    )
    agent2_wins = sum(
        1 for w, _ in results if ("agent2" in w.lower() or "black" in w.lower())
    )
    draws = sum(1 for w, _ in results if w == "draw.")

    print(f"Agent1 Wins: {agent1_wins}")
    print(f"Agent2 Wins: {agent2_wins}")
    print(f"Draws: {draws}")

    for res in results:
        print(f"Result: {res[0]}")
        print(f"Game string: {res[1]}")
        print()


if __name__ == "__main__":
    import concurrent.futures

    main()
