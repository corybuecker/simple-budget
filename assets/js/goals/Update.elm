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
    case model.activeGoal of
        Just oldActiveGoal ->
            case msg of
                Goals.Messages.TitleUpdated newName ->
                    let
                        newActiveGoal =
                            { oldActiveGoal | title = newName }
                    in
                    ( { model | activeGoal = Just newActiveGoal }, Cmd.none )

                Goals.Messages.StartDateUpdated newStartDate ->
                    let
                        newActiveGoal =
                            { oldActiveGoal | startDate = newStartDate }
                    in
                    ( { model | activeGoal = Just newActiveGoal }, Cmd.none )

                Goals.Messages.EndDateUpdated newEndDate ->
                    let
                        newActiveGoal =
                            { oldActiveGoal | endDate = newEndDate }
                    in
                    ( { model | activeGoal = Just newActiveGoal }, Cmd.none )

                Goals.Messages.TargetUpdated newTarget ->
                    let
                        newActiveGoal =
                            case String.toFloat newTarget of
                                Just val ->
                                    { oldActiveGoal | target = val }

                                Nothing ->
                                    { oldActiveGoal | target = 0 }
                    in
                    ( { model | activeGoal = Just newActiveGoal }, Cmd.none )

                Goals.Messages.SaveGoal ->
                    ( model, saveGoalAndRefreshGoals oldActiveGoal )

                Goals.Messages.DeleteGoal ->
                    ( model, deleteGoalAndRefreshGoals oldActiveGoal )

        Nothing ->
            ( model, Cmd.none )


fetchGoals : Cmd Msg
fetchGoals =
    Http.send GoalsFetched (get goalsUrl goalsDecoder)


deleteGoalAndRefreshGoals : Goals.Models.Goal -> Cmd Msg
deleteGoalAndRefreshGoals model =
    Task.attempt GoalsFetched (Task.andThen refreshGoalsTask (deleteGoalTask model))


saveGoalAndRefreshGoals : Goals.Models.Goal -> Cmd Msg
saveGoalAndRefreshGoals model =
    case model.id of
        0 ->
            Task.attempt GoalsFetched (Task.andThen refreshGoalsTask (saveNewGoalTask model))

        _ ->
            Task.attempt GoalsFetched (Task.andThen refreshGoalsTask (saveGoalTask model))


refreshGoalsTask : a -> Task Http.Error (List Goal)
refreshGoalsTask _ =
    toTask (get goalsUrl goalsDecoder)


saveGoalTask : Goals.Models.Goal -> Task Http.Error Goal
saveGoalTask model =
    toTask (put (goalUrl model.id) (jsonBody (encode model)) goalUpdatedDecoder)


deleteGoalTask : Goals.Models.Goal -> Task Http.Error String
deleteGoalTask model =
    toTask (delete (goalUrl model.id))


saveNewGoalTask : Goals.Models.Goal -> Task Http.Error Goal
saveNewGoalTask model =
    toTask (post goalsUrl (jsonBody (encode model)) goalUpdatedDecoder)


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


delete : String -> Http.Request String
delete url =
    Http.request
        { method = "DELETE"
        , headers = []
        , url = url
        , body = Http.emptyBody
        , expect = Http.expectString
        , timeout = Nothing
        , withCredentials = False
        }
