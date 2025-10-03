use futures::StreamExt;
use std::time::Duration;
use tutti_core::{CommandSpec, ProcessManager, UnixProcessManager};

#[tokio::test]
#[cfg(unix)]
async fn test_process_manager_stdout() {
    let mut pm = UnixProcessManager::new();

    let out = pm
        .spawn(CommandSpec {
            name: "stdout".to_owned(),
            cmd: vec!["bash".to_owned(), "./stdout.sh".to_owned()],
            cwd: Some("./tests/fixtures/".parse().unwrap()),
            env: vec![],
        })
        .await
        .unwrap();

    let mut stdout = out.stdout;
    {
        let expected_stdout = "INFO: line 1\nINFO: line 2\nINFO: stdout.sh finished\n".to_owned();
        let mut actual_stdout = String::new();

        while let Some(line) = stdout.next().await {
            actual_stdout.push_str(&String::from_utf8_lossy(&line));
        }
        assert_eq!(expected_stdout, actual_stdout);
    }

    let mut stderr = out.stderr;
    {
        let expected_stderr = String::new();
        let mut actual_stderr = String::new();

        while let Some(line) = stderr.next().await {
            actual_stderr.push_str(&String::from_utf8_lossy(&line));
        }
        assert_eq!(expected_stderr, actual_stderr);
    }
}

#[tokio::test]
#[cfg(unix)]
async fn test_process_manager_stderr() {
    let mut pm = UnixProcessManager::new();

    let out = pm
        .spawn(CommandSpec {
            name: "stderr".to_owned(),
            cmd: vec!["bash".to_owned(), "./stderr.sh".to_owned()],
            cwd: Some("./tests/fixtures/".parse().unwrap()),
            env: vec![],
        })
        .await
        .unwrap();

    let mut stdout = out.stdout;
    {
        let expected_stdout = String::new();
        let mut actual_stdout = String::new();

        while let Some(line) = stdout.next().await {
            actual_stdout.push_str(&String::from_utf8_lossy(&line));
        }
        assert_eq!(expected_stdout, actual_stdout);
    }

    let mut stderr = out.stderr;
    {
        let expected_stderr =
            "ERROR: line 1\nERROR: line 2\nERROR: stderr.sh finished\n".to_owned();
        let mut actual_stderr = String::new();

        while let Some(line) = stderr.next().await {
            actual_stderr.push_str(&String::from_utf8_lossy(&line));
        }
        assert_eq!(expected_stderr, actual_stderr);
    }
}

#[tokio::test]
#[cfg(unix)]
async fn test_process_manager_both() {
    let mut pm = UnixProcessManager::new();

    let out = pm
        .spawn(CommandSpec {
            name: "both".to_owned(),
            cmd: vec!["bash".to_owned(), "./both.sh".to_owned()],
            cwd: Some("./tests/fixtures/".parse().unwrap()),
            env: vec![],
        })
        .await
        .unwrap();

    let mut stdout = out.stdout;
    {
        let expected_stdout =
            "STDOUT: message 2\nSTDOUT: message 4\nSTDOUT: message 6\nSTDOUT: message 8\nboth.sh done\n".to_owned();
        let mut actual_stdout = String::new();

        while let Some(line) = stdout.next().await {
            actual_stdout.push_str(&String::from_utf8_lossy(&line));
        }
        assert_eq!(expected_stdout, actual_stdout);
    }

    let mut stderr = out.stderr;
    {
        let expected_stderr =
            "STDERR: message 1\nSTDERR: message 3\nSTDERR: message 5\nSTDERR: message 7\n"
                .to_owned();
        let mut actual_stderr = String::new();

        while let Some(line) = stderr.next().await {
            actual_stderr.push_str(&String::from_utf8_lossy(&line));
        }
        assert_eq!(expected_stderr, actual_stderr);
    }
}

#[tokio::test]
#[cfg(unix)]
async fn test_process_manager_sigint() {
    let mut pm = UnixProcessManager::new();

    let out = pm
        .spawn(CommandSpec {
            name: "infinite_sigint_kills".to_owned(),
            cmd: vec!["bash".to_owned(), "./infinite_sigint_kills.sh".to_owned()],
            cwd: Some("./tests/fixtures/".parse().unwrap()),
            env: vec![],
        })
        .await
        .unwrap();

    pm.shutdown(out.id).await.unwrap();
    let result = pm.wait(out.id, Duration::from_millis(100)).await.unwrap();
    assert_eq!(result, Some(0));
}

#[tokio::test]
#[cfg(unix)]
async fn test_process_manager_sigkill() {
    let mut pm = UnixProcessManager::new();

    let out = pm
        .spawn(CommandSpec {
            name: "ignore_sigint_sigkill_only".to_owned(),
            cmd: vec![
                "bash".to_owned(),
                "./ignore_sigint_sigkill_only.sh".to_owned(),
            ],
            cwd: Some("./tests/fixtures/".parse().unwrap()),
            env: vec![],
        })
        .await
        .unwrap();

    pm.shutdown(out.id).await.unwrap();
    let result = pm.wait(out.id, Duration::from_millis(100)).await.unwrap();
    assert_eq!(result, None);
    pm.kill(out.id).await.unwrap();
    let result = pm.wait(out.id, Duration::from_millis(100)).await.unwrap();
    assert_eq!(result, Some(0));
}
