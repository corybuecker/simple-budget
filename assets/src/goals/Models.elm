module Goals.Models exposing (Goal)


type alias Goal =
    { id : Int
    , title : String
    , startDate : String
    , endDate : String
    , target : Float
    }
