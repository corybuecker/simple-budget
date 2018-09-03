module Goals.Models exposing (..)


type alias Goal =
    { id : Int
    , title : String
    , startDate : String
    , endDate : String
    , target : Float
    }
