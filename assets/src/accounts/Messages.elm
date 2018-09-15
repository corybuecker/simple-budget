module Accounts.Messages exposing (Msg(..))

import Accounts.Models exposing (Account)
import Http


type Msg
    = NameUpdated String
    | DebtUpdated Bool
    | BalanceUpdated Float
