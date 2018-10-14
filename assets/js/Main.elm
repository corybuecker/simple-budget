module Main exposing (errorMessage, init, main, modalView, subscriptions, update, updatePage, view)

import Accounts.Messages
import Accounts.Models
import Accounts.Update
import Accounts.Utils exposing (accountDecoder, accountsDecoder, adjustmentDecoder)
import Accounts.Views
import Adjustments.Models
import Adjustments.Update
import Adjustments.Views
import Browser
import Browser.Navigation
import Debug exposing (log)
import Goals.Models
import Goals.Update
import Goals.Views
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode as Decode
import Json.Decode.Pipeline exposing (hardcoded, optional, required)
import List exposing (..)
import Model exposing (Model, Msg(..))
import Savings.Models
import Savings.Update
import Savings.Views
import String
import Url exposing (Url)
import Url.Builder as Url



-- MAIN


main =
    Browser.application
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        , onUrlChange = Model.UrlChanged
        , onUrlRequest = Model.UrlRequest
        }



-- MODEL


init : () -> Url -> Browser.Navigation.Key -> ( Model, Cmd Msg )
init _ url key =
    let
        emptyModel =
            Model [] [] [] Nothing "" (Just Accounts.Models.newAccount) (Just Goals.Models.newGoal) (Just Savings.Models.newSaving) Nothing key ""
    in
    updatePage url emptyModel



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        AccountsFetched result ->
            case result of
                Ok accounts ->
                    ( { model | accounts = accounts, activeAccount = Nothing }
                    , Cmd.none
                    )

                Err error ->
                    ( { model | error = Just error }
                    , Cmd.none
                    )

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

        SavingsFetched result ->
            case result of
                Ok savings ->
                    ( { model | savings = savings, activeSaving = Nothing }
                    , Cmd.none
                    )

                Err error ->
                    ( { model | error = Just error }
                    , Cmd.none
                    )

        AdjustmentsFetched result ->
            case result of
                Ok adjustments ->
                    ( model
                    , Cmd.none
                    )

                Err error ->
                    ( { model | error = Just error }
                    , Cmd.none
                    )

        OpenAccountEditor account ->
            ( { model | modalOpen = "account", activeAccount = Just account }, Cmd.none )

        OpenGoalEditor goal ->
            ( { model | modalOpen = "goal", activeGoal = Just goal }, Cmd.none )

        OpenSavingEditor saving ->
            ( { model | modalOpen = "saving", activeSaving = Just saving }, Cmd.none )

        OpenAdjustmentEditor adjustment ->
            ( { model | modalOpen = "adjustment", activeAdjustment = Just adjustment }, Cmd.none )

        UpdateAccount accountMsg ->
            Accounts.Update.update accountMsg model

        UpdateGoal goalMsg ->
            Goals.Update.update goalMsg model

        UpdateSaving savingMsg ->
            Savings.Update.update savingMsg model

        UpdateAdjustment adjustmentMsg ->
            Adjustments.Update.update adjustmentMsg model

        CreateAccount ->
            ( { model | modalOpen = "account", activeAccount = Just Accounts.Models.newAccount }, Cmd.none )

        CreateAdjustment ->
            ( { model | modalOpen = "adjustment", activeAdjustment = Just Adjustments.Models.newAdjustment }, Cmd.none )

        CreateGoal ->
            ( { model | modalOpen = "goal", activeGoal = Just Goals.Models.newGoal }, Cmd.none )

        CreateSaving ->
            ( { model | modalOpen = "saving", activeSaving = Just Savings.Models.newSaving }, Cmd.none )

        UrlChanged url ->
            updatePage url model

        UrlRequest request ->
            case request of
                Browser.Internal url ->
                    let
                        key =
                            model.key

                        urlString =
                            Url.toString url
                    in
                    ( model, Browser.Navigation.pushUrl key urlString )

                _ ->
                    ( model, Cmd.none )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


updatePage : Url -> Model -> ( Model, Cmd Msg )
updatePage url model =
    case url.path of
        "/accounts" ->
            ( { model | page = "accounts" }, Accounts.Update.fetchAccounts )

        "/goals" ->
            ( { model | page = "goals" }, Goals.Update.fetchGoals )

        "/savings" ->
            ( { model | page = "savings" }, Savings.Update.fetchSavings )

        _ ->
            ( { model | page = "home" }, Cmd.none )



-- VIEW


modalView : Model -> Html Msg
modalView model =
    case model.modalOpen of
        "account" ->
            case model.activeAccount of
                Just a ->
                    Html.map UpdateAccount (Accounts.Views.editView a)

                Nothing ->
                    div [] []

        "goal" ->
            case model.activeGoal of
                Just a ->
                    Html.map UpdateGoal (Goals.Views.editView a)

                Nothing ->
                    div [] []

        "saving" ->
            case model.activeSaving of
                Just a ->
                    Html.map UpdateSaving (Savings.Views.editView a)

                Nothing ->
                    div [] []

        _ ->
            div [] []


view : Model -> Browser.Document Msg
view model =
    let
        body =
            case model.page of
                "accounts" ->
                    Accounts.Views.renderAccounts model.accounts

                "goals" ->
                    Goals.Views.renderGoals model.goals

                "savings" ->
                    Savings.Views.renderSavings model.savings

                _ ->
                    Accounts.Views.renderAccounts model.accounts
    in
    { title = "test"
    , body =
        [ div []
            [ a [ href "/accounts" ] [ text "Accounts" ]
            , a [ href "/goals" ] [ text "Goals" ]
            , a [ href "/savings" ] [ text "Savings" ]
            , body
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



-- HTTP
