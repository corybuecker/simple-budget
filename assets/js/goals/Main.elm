module Main exposing (errorMessage, init, main, subscriptions, update, view)

import Browser
import Goals.Messages exposing (..)
import Goals.Models exposing (..)
import Goals.Utils exposing (..)
import Goals.Views exposing (..)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http exposing (get, jsonBody, post)
import Json.Decode
import Json.Decode.Pipeline exposing (hardcoded, optional, required)
import List exposing (..)
import String
import Task exposing (Task)
import Url exposing (Url)
import Url.Builder as Url


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( emptyModel
    , fetchGoals
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GoalsFetched result ->
            case result of
                Ok goals ->
                    ( { model | goals = goals, activeGoal = Nothing }
                    , Cmd.none
                    )

                Err error ->
                    ( { model | error = Just error }
                    , Cmd.none
                    )

        TitleUpdated newName ->
            case model.activeGoal of
                Just oldActiveGoal ->
                    let
                        newActiveGoal =
                            { oldActiveGoal | title = newName }
                    in
                    ( { model | activeGoal = Just newActiveGoal }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        TargetUpdated newTotal ->
            case model.activeGoal of
                Just oldActiveGoal ->
                    let
                        newActiveGoal =
                            case String.toFloat newTotal of
                                Just val ->
                                    { oldActiveGoal | target = val }

                                Nothing ->
                                    { oldActiveGoal | target = 0 }
                    in
                    ( { model | activeGoal = Just newActiveGoal }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        OpenGoalEditor goal ->
            ( { model | modalOpen = "goal", activeGoal = Just goal }, Cmd.none )

        CreateGoal ->
            ( { model | activeGoal = Just Goals.Models.newGoal }, Cmd.none )

        SaveGoal ->
            case model.activeGoal of
                Just oldActiveGoal ->
                    ( model, saveGoalAndRefreshGoals oldActiveGoal )

                Nothing ->
                    ( model, Cmd.none )

        DeleteGoal ->
            case model.activeGoal of
                Just oldActiveGoal ->
                    ( model, deleteGoalAndRefreshGoal oldActiveGoal )

                Nothing ->
                    ( model, Cmd.none )

        StartDateUpdated newStartDate ->
            case model.activeGoal of
                Just oldActiveGoal ->
                    let
                        newActiveGoal =
                            { oldActiveGoal | startDate = newStartDate }
                    in
                    ( { model | activeGoal = Just newActiveGoal }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        EndDateUpdated newEndDate ->
            case model.activeGoal of
                Just oldActiveGoal ->
                    let
                        newActiveGoal =
                            { oldActiveGoal | endDate = newEndDate }
                    in
                    ( { model | activeGoal = Just newActiveGoal }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        DeleteCompleted result ->
            ( model, fetchGoals )

        GoalSaved result ->
            ( model, fetchGoals )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


modalView : Model -> Html Msg
modalView model =
    case model.activeGoal of
        Just a ->
            editView a

        Nothing ->
            div [] []


view : Model -> Html Msg
view model =
    div []
        [ div [] [ renderGoals model.goals ]
        , div [] [ modalView model ]
        , p [] [ text (errorMessage model.error) ]
        ]


errorMessage : Maybe Http.Error -> String
errorMessage error =
    case error of
        Nothing ->
            ""

        Just (Http.BadBody message) ->
            message

        _ ->
            "Unknown"


deleteGoalAndRefreshGoal : Goal -> Cmd Msg
deleteGoalAndRefreshGoal model =
    deleteGoalTask model


fetchGoals : Cmd Msg
fetchGoals =
    get goalsUrl GoalsFetched goalsDecoder


saveGoalAndRefreshGoals : Goal -> Cmd Msg
saveGoalAndRefreshGoals model =
    case model.id of
        0 ->
            saveNewGoalTask model

        _ ->
            saveGoalTask model


saveNewGoalTask : Goal -> Cmd Msg
saveNewGoalTask model =
    post goalsUrl (jsonBody (Goals.Utils.encode model)) GoalSaved goalUpdatedDecoder


refreshGoalsTask : a -> Cmd Msg
refreshGoalsTask _ =
    get goalsUrl GoalsFetched goalsDecoder


saveGoalTask : Goal -> Cmd Msg
saveGoalTask model =
    put (goalUrl model.id) (jsonBody (Goals.Utils.encode model)) GoalSaved goalUpdatedDecoder


deleteGoalTask : Goal -> Cmd Msg
deleteGoalTask model =
    delete (goalUrl model.id)


goalsUrl : String
goalsUrl =
    Url.relative
        [ "api", "goals" ]
        []


goalUrl : Int -> String
goalUrl id =
    Url.relative
        [ "api", "goals", String.fromInt id ]
        []


get : String -> (Result Http.Error a -> Msg) -> Json.Decode.Decoder a -> Cmd Msg
get url msg decoder =
    Http.request
        { method = "GET"
        , headers = []
        , url = url
        , body = Http.emptyBody
        , expect = Http.expectJson msg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


put : String -> Http.Body -> (Result Http.Error a -> Msg) -> Json.Decode.Decoder a -> Cmd Msg
put url body msg decoder =
    Http.request
        { method = "PUT"
        , headers = []
        , url = url
        , body = body
        , expect = Http.expectJson msg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


post : String -> Http.Body -> (Result Http.Error a -> Msg) -> Json.Decode.Decoder a -> Cmd Msg
post url body msg decoder =
    Http.request
        { method = "POST"
        , headers = []
        , url = url
        , body = body
        , expect = Http.expectJson msg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


delete : String -> Cmd Msg
delete url =
    Http.request
        { method = "DELETE"
        , headers = []
        , url = url
        , body = Http.emptyBody
        , expect = Http.expectWhatever DeleteCompleted
        , timeout = Nothing
        , tracker = Nothing
        }
