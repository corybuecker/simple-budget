module Main exposing (..)

import Browser
import Browser.Navigation
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode as Decode
import Url.Builder as Url
import List exposing (..)
import String
import Json.Decode.Pipeline exposing (required, optional, hardcoded)
import Accounts.Utils exposing (accountsDecoder, accountDecoder, adjustmentDecoder)
import Accounts.Models exposing (Account)
import Goals.Models exposing (Goal)
import Accounts.Views
import Goals.Views
import Accounts.Messages
import Accounts.Update
import Goals.Update
import Model exposing (Model, Msg(..))
import Url exposing (Url)
import Debug exposing (log)


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
    ( Model [] [] Nothing "" Accounts.Views.emptyAccount Goals.Views.emptyGoal key "home"
    , Accounts.Update.fetchAccounts
    )



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        AccountsFetched result ->
            case result of
                Ok accounts ->
                    ( { model | accounts = accounts }
                    , Cmd.none
                    )

                Err error ->
                    ( { model | error = Just error }
                    , Cmd.none
                    )

        GoalsFetched result ->
            case result of
                Ok goals ->
                    ( { model | goals = goals }
                    , Cmd.none
                    )

                Err error ->
                    ( { model | error = Just error }
                    , Cmd.none
                    )

        OpenAccountEditor account ->
            ( { model | modalOpen = "account", activeAccount = account }, Cmd.none )

        OpenGoalEditor goal ->
            ( { model | modalOpen = "goal", activeGoal = goal }, Cmd.none )

        UpdateAccount accountMsg ->
            Accounts.Update.update accountMsg model

        UpdateGoal goalMsg ->
            Goals.Update.update goalMsg model

        UrlChanged url ->
            case url.path of
                "/accounts" ->
                    ( { model | page = "accounts" }, Accounts.Update.fetchAccounts )

                "/goals" ->
                    ( { model | page = "goals" }, Goals.Update.fetchGoals )

                _ ->
                    ( { model | page = "home" }, Cmd.none )

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



-- VIEW


modalView : Model -> Html Msg
modalView model =
    case model.modalOpen of
        "account" ->
            Html.map UpdateAccount (Accounts.Views.editView model.activeAccount)

        "goal" ->
            Html.map UpdateGoal (Goals.Views.editView model.activeGoal)

        _ ->
            div [] []


view : Model -> Browser.Document Msg
view model =
    let
        body =
            case model.page of
                "accounts" ->
                    renderAccounts model

                "goals" ->
                    renderGoals model

                _ ->
                    renderAccounts model
    in
        { title = "test"
        , body =
            [ div []
                [ a [ href "/accounts" ] [ text "Accounts" ]
                , a [ href "/goals" ] [ text "Goals" ]
                , body
                , div [] [ modalView model ]
                , p [] [ text (errorMessage model.error) ]
                ]
            ]
        }


renderAccounts : Model -> Html Msg
renderAccounts model =
    table []
        [ thead []
            [ tr []
                [ th [] [ text "Account Name" ]
                , th [] [ text "Balance" ]
                , th [] [ text "Debt?" ]
                ]
            ]
        , tbody [] (List.map renderAccount model.accounts)
        ]


renderGoals : Model -> Html Msg
renderGoals model =
    table []
        [ thead []
            [ tr []
                [ th [] [ text "Account Name" ]
                , th [] [ text "Balance" ]
                , th [] [ text "Debt?" ]
                ]
            ]
        , tbody [] (List.map renderGoal model.goals)
        ]


renderGoal : Goal -> Html Msg
renderGoal goal =
    tr []
        [ td [ onClick (OpenGoalEditor goal) ] [ text goal.title ]
        , td [] [ text (String.fromFloat goal.target) ]
        , td [] [ text goal.startDate ]
        , td [] [ text goal.endDate ]
        ]


renderAccount : Account -> Html Msg
renderAccount account =
    tr []
        [ td [ onClick (OpenAccountEditor account) ] [ text account.name ]
        , td [] [ text (String.fromFloat account.balance) ]
        , td []
            [ text
                (if account.debt then
                    "True"
                 else
                    "False"
                )
            ]
        ]


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
