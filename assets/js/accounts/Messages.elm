module Accounts.Messages exposing (Msg(..))


type Msg
    = NameUpdated String
    | ToggleDebt
    | BalanceUpdated String
    | SaveAccount
    | DeleteAccount
