# TODO

- or
- transaction
- 同じテーブルを複数回 join
- 無条件に distinct 付けるのやめる

# DONE

- order("users.id") -> order().id().asc() みたいにしたい
- has_one
- User::select().roles(|role| role.preload) だけなら where に roles は出てこないので join しない。
  これ join 必要だった。join ないと roles のない users も返ってくるから
  って、それでよかった。
  preload だけの指定は N+1 対策なので join しなくていいの。
- join したテーブルの order by. preload の時に使う
  Error: db error: ERROR: SELECT DISTINCTではORDER BYの式はSELECTリスト内になければなりません
  `.create_projects(|x| x.preload().id().eq(1).order().id().asc())`
