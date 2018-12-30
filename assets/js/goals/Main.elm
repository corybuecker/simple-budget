module Main exposing (errorMessage, init, main, subscriptions, update, view)

import Browser
import Goals.Messages exposing (..)
import Goals.Models exposing (..)
import Goals.Utils exposing (..)
import Goals.Views exposing (..)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http exposing (get, jsonBody, post, toTask)
import Json.Decode
import Json.Decode.Pipeline exposing (hardcoded, optional, required)
import List exposing (..)
import String
import Task exposing (Task)
import Url exposing (Url)
import Url.Builder as Url


main =
    Browser.document
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


view : Model -> Browser.Document Msg
view model =
    { title = "Simple Budget"
    , body =
        [ header [ class "navbar navbar-expand-lg navbar-light bg-light" ]
            [ a [ class "navbar-brand", href "/home" ] [ text "Simple Budget" ]
            , ul [ class "navbar-nav mr-auto" ]
                [ li [ class "nav-item" ] [ a [ class "nav-link", href "/goals" ] [ text "Goals" ] ]
                , li [ class "nav-item" ] [ a [ class "nav-link", href "/goals" ] [ text "Goals" ] ]
                , li [ class "nav-item" ] [ a [ class "nav-link", href "/savings" ] [ text "Savings" ] ]
                ]
            ]
        , div [ class "container-fluid" ]
            [ div [ class "row flex-xl-nowrap" ] []
            , div [] [ renderGoals model.goals ]
            , div [] [ modalView model ]
            , p [] [ text (errorMessage model.error) ]
            ]
        ]
    }


errorMessage : Maybe Http.Error -> String
errorMessage error =
    case error of
        Nothing ->
            ""

        Just (Http.BadPayload message _) ->
            message

        _ ->
            "Unknown"


deleteGoalAndRefreshGoal : Goal -> Cmd Msg
deleteGoalAndRefreshGoal model =
    Task.attempt GoalsFetched (Task.andThen refreshGoalsTask (deleteGoalTask model))


fetchGoals : Cmd Msg
fetchGoals =
    Http.send GoalsFetched (get goalsUrl goalsDecoder)


saveGoalAndRefreshGoals : Goal -> Cmd Msg
saveGoalAndRefreshGoals model =
    case model.id of
        0 ->
            Task.attempt GoalsFetched (Task.andThen refreshGoalsTask (saveNewGoalTask model))

        _ ->
            Task.attempt GoalsFetched (Task.andThen refreshGoalsTask (saveGoalTask model))


saveNewGoalTask : Goal -> Task Http.Error Goal
saveNewGoalTask model =
    toTask (post goalsUrl (jsonBody (Goals.Utils.encode model)) goalUpdatedDecoder)


refreshGoalsTask : a -> Task Http.Error (List Goal)
refreshGoalsTask _ =
    toTask (get goalsUrl goalsDecoder)


saveGoalTask : Goal -> Task Http.Error Goal
saveGoalTask model =
    toTask (put (goalUrl model.id) (jsonBody (Goals.Utils.encode model)) goalUpdatedDecoder)


deleteGoalTask : Goal -> Task Http.Error String
deleteGoalTask model =
    toTask (delete (goalUrl model.id))


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
