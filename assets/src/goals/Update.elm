module Goals.Update exposing (..)

import Goals.Messages
import Goals.Models exposing (Goal)
import Goals.Views
import Model exposing (Model, Msg(..))
import Http exposing (post, toTask, jsonBody, get)
import Goals.Utils exposing (goalsDecoder, goalDecoder, goalUpdatedDecoder)
import Url.Builder as Url
import Goals.Utils exposing (encode)
import Task exposing (Task)
import Json.Decode


update : Goals.Messages.Msg -> Model -> ( Model, Cmd Model.Msg )
update msg model =
    case msg of
        Goals.Messages.NameUpdated newName ->
            let
                oldActiveGoal =
                    model.activeGoal

                newActiveGoal =
                    { oldActiveGoal | title = newName }
            in
                ( { model | activeGoal = newActiveGoal }, saveGoalAndRefreshGoals newActiveGoal )

        _ ->
            ( model, Cmd.none )


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
