module Goals.Messages exposing (..)

import Goals.Models exposing (Goal)
import Http


type Msg
    = NameUpdated String
    | DebtUpdated Bool
    | BalanceUpdated Float
