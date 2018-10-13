module Accounts.Models exposing (Account, Adjustment, newAccount)


type alias Adjustment =
    { id : Int
    , title : String
    , total : Float
    }


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
