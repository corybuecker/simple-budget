module Savings.Models exposing (Saving, newSaving)


newSaving : Saving
newSaving =
    Saving 0 "" 0.0


type alias Saving =
    { id : Int
    , title : String
    , amount : Float
    }
