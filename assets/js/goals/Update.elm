module Goals.Update exposing (fetchGoals, goalUrl, goalsUrl, put, refreshGoalsTask, saveGoalAndRefreshGoals, saveGoalTask, update)

import Goals.Messages
import Goals.Models exposing (Goal)
import Goals.Utils exposing (encode, goalDecoder, goalUpdatedDecoder, goalsDecoder)
import Goals.Views
import Http exposing (get, jsonBody, post, toTask)
import Json.Decode
import Model exposing (Model, Msg(..))
import Task exposing (Task)
import Url.Builder as Url


update : Goals.Messages.Msg -> Model -> ( Model, Cmd Model.Msg )
update msg model =
    case msg of
        Goals.Messages.TitleUpdated newName ->
            let
                oldActiveGoal =
                    model.activeGoal

                newActiveGoal =
                    { oldActiveGoal | title = newName }
            in
            ( { model | activeGoal = newActiveGoal }, saveGoalAndRefreshGoals newActiveGoal )

        Goals.Messages.StartDateUpdated newStartDate ->
            let
                oldActiveGoal =
                    model.activeGoal

                newActiveGoal =
                    { oldActiveGoal | startDate = newStartDate }
            in
            ( { model | activeGoal = newActiveGoal }, saveGoalAndRefreshGoals newActiveGoal )

        Goals.Messages.EndDateUpdated newEndDate ->
            let
                oldActiveGoal =
                    model.activeGoal

                newActiveGoal =
                    { oldActiveGoal | endDate = newEndDate }
            in
            ( { model | activeGoal = newActiveGoal }, saveGoalAndRefreshGoals newActiveGoal )

        Goals.Messages.TargetUpdated newTarget ->
            let
                oldActiveGoal =
                    model.activeGoal

                newActiveGoal =
                    case String.toFloat newTarget of
                        Just val ->
                            { oldActiveGoal | target = val }

                        Nothing ->
                            { oldActiveGoal | target = 0 }
            in
            ( { model | activeGoal = newActiveGoal }, saveGoalAndRefreshGoals newActiveGoal )


fetchGoals : Cmd Msg
fetchGoals =
    Http.send GoalsFetched (get goalsUrl goalsDecoder)


saveGoalAndRefreshGoals : Goals.Models.Goal -> Cmd Msg
saveGoalAndRefreshGoals model =
    Task.attempt GoalsFetched (Task.andThen refreshGoalsTask (saveGoalTask model))


refreshGoalsTask : Goal -> Task Http.Error (List Goal)
refreshGoalsTask _ =
    toTask (get goalsUrl goalsDecoder)


saveGoalTask : Goals.Models.Goal -> Task Http.Error Goal
saveGoalTask model =
    toTask (put (goalUrl model.id) (jsonBody (encode model)) goalUpdatedDecoder)


goalsUrl : String
goalsUrl =
    Url.crossOrigin "//localhost:4000"
        [ "api", "goals" ]
        []


goalUrl : Int -> String
goalUrl id =
    Url.crossOrigin "//localhost:4000"
        [ "api", "goals", String.fromInt id ]
        []


put : String -> Http.Body -> Json.Decode.Decoder a -> Http.Request a
put url body decoder =
    Http.request
        { method = "PUT"
        , headers = []
        , url = url
        , body = body
        , expect = Http.expectJson decoder
        , timeout = Nothing
        , withCredentials = False
        }
