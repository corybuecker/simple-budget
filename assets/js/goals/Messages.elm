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
    | CreateGoal
    | OpenGoalEditor Goal
    | GoalsFetched (Result Http.Error (List Goal))
    | DeleteCompleted (Result Http.Error ())
    | GoalSaved (Result Http.Error Goal)
