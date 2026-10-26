push message:
    git add . && git commit -m "{{message}}" && git push

migrate-up name:
    sqlx migrate add -r {{name}}

migrate-run:
    sqlx migrate run