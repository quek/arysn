``` emacs-lisp
;;(setq lsp-rust-rls-server-command '("docker" "exec" "-i" "arysn_dev_1" "rls" "--cli"))
(setq lsp-rust-rls-server-command '("docker" "exec" "-i" "arysn_dev_1" "rls"))
```

`docker exec -i arysn_dev_1 rls`

TODO プロジェクトルートの .dir-locals.el で上の同じようなのを設定する
