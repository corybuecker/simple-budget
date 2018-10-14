module Goals.Messages exposing (Msg(..))

import Goals.Models exposing (Goal)
import Http


type Msg
    = TitleUpdated String
    | TargetUpdated String
    | StartDateUpdated String
    | EndDateUpdated String
    | SaveGoal
    | DeleteGoal
