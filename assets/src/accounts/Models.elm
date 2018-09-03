module Accounts.Models exposing (..)


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
