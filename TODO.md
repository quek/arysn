# TODO

- 無条件に distinct 付けるのやめる
- order("users.id") -> order().id().asc() みたいにしたい

# DONE

- User::select().roles(|role| role.preload) だけなら where に roles は出てこないので join しない。
  これ join 必要だった。join ないと roles のない users も返ってくるから
