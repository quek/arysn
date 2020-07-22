# TODO

- join したテーブルの order by. preload の時に使う？
  Error: db error: ERROR: SELECT DISTINCTではORDER BYの式はSELECTリスト内になければなりません
  `.create_projects(|x| x.preload().id().eq(1).order().id().asc())`
- 同じテーブルを複数回 join
- 無条件に distinct 付けるのやめる

# DONE

- User::select().roles(|role| role.preload) だけなら where に roles は出てこないので join しない。
  これ join 必要だった。join ないと roles のない users も返ってくるから
- order("users.id") -> order().id().asc() みたいにしたい
