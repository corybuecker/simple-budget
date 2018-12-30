module Goals.Models exposing (Goal, Model, emptyModel, newGoal)

import Http


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


type alias Model =
    { goals : List Goal
    , activeGoal : Maybe Goal
    , error : Maybe Http.Error
    , modalOpen : String
    }


emptyModel : Model
emptyModel =
    Model [] Nothing Nothing ""
