module Goals.Models exposing (Goal, newGoal)


newGoal : Goal
newGoal =
    Goal 0 "" "" "" 0.0


type alias Goal =
    { id : Int
    , title : String
    , startDate : String
    , endDate : String
    , target : Float
    }
