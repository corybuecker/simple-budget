module Accounts.Messages exposing (..)

import Accounts.Models exposing (Account)
import Http


type Msg
    = NameUpdated String
    | DebtUpdated Bool
    | BalanceUpdated Float
