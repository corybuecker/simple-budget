module Accounts.Models exposing (Account, newAccount)

import Adjustments.Models exposing (Adjustment)


type alias Account =
    { id : Int
    , name : String
    , debt : Bool
    , balance : Float
    , adjustments : List Adjustment
    }


newAccount : Account
newAccount =
    Account 0 "" False 0.0 []
