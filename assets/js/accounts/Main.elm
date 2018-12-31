module Main exposing (errorMessage, init, main, subscriptions, update, view)

import Accounts.Messages exposing (..)
import Accounts.Models exposing (..)
import Accounts.Utils exposing (..)
import Accounts.Views exposing (..)
import Adjustments.Utils exposing (..)
import Browser
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
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( emptyModel
    , fetchAccounts
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        AccountsFetched result ->
            case result of
                Ok accounts ->
                    ( { model | accounts = accounts, activeAccount = Nothing, activeAdjustment = Nothing }
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

        TitleUpdated newName ->
            case model.activeAdjustment of
                Just oldActiveAdjustment ->
                    let
                        newActiveAdjustment =
                            { oldActiveAdjustment | title = newName }
                    in
                    ( { model | activeAdjustment = Just newActiveAdjustment }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        TotalUpdated newTotal ->
            case model.activeAdjustment of
                Just oldActiveAdjustment ->
                    let
                        newActiveAdjustment =
                            case String.toFloat newTotal of
                                Just val ->
                                    { oldActiveAdjustment | total = val }

                                Nothing ->
                                    { oldActiveAdjustment | total = 0 }
                    in
                    ( { model | activeAdjustment = Just newActiveAdjustment }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        SaveAdjustment ->
            case model.activeAdjustment of
                Just oldActiveAdjustment ->
                    ( model, saveAdjustmentAndRefreshAdjustments oldActiveAdjustment )

                Nothing ->
                    ( model, Cmd.none )

        DeleteAdjustment ->
            case model.activeAdjustment of
                Just oldActiveAdjustment ->
                    ( model, deleteAdjustmentAndRefreshAdjustments oldActiveAdjustment )

                Nothing ->
                    ( model, Cmd.none )

        OpenAccountEditor account ->
            ( { model | modalOpen = "account", activeAccount = Just account }, Cmd.none )

        OpenAdjustmentEditor adjustment ->
            ( { model | modalOpen = "adjustment", activeAdjustment = Just adjustment }, Cmd.none )

        CreateAccount ->
            ( { model | modalOpen = "account", activeAccount = Just newAccount }, Cmd.none )

        CreateAdjustment account ->
            let
                newAdjustment =
                    Adjustment account.id 0 "" 0.0
            in
            ( { model | modalOpen = "adjustment", activeAdjustment = Just newAdjustment }, Cmd.none )

        ToggleDebt ->
            case model.activeAccount of
                Just oldActiveAccount ->
                    let
                        newActiveAccount =
                            { oldActiveAccount | debt = not oldActiveAccount.debt }
                    in
                    ( { model | activeAccount = Just newActiveAccount }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        NameUpdated newName ->
            case model.activeAccount of
                Just oldActiveAccount ->
                    let
                        newActiveAccount =
                            { oldActiveAccount | name = newName }
                    in
                    ( { model | activeAccount = Just newActiveAccount }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        BalanceUpdated newBalance ->
            case model.activeAccount of
                Just oldActiveAccount ->
                    let
                        newActiveAccount =
                            case String.toFloat newBalance of
                                Just val ->
                                    { oldActiveAccount | balance = val }

                                Nothing ->
                                    { oldActiveAccount | balance = 0 }
                    in
                    ( { model | activeAccount = Just newActiveAccount }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        SaveAccount ->
            case model.activeAccount of
                Just oldActiveAccount ->
                    ( model, saveAccountAndRefreshAccounts oldActiveAccount )

                Nothing ->
                    ( model, Cmd.none )

        DeleteAccount ->
            case model.activeAccount of
                Just oldActiveAccount ->
                    ( model, deleteAccountAndRefreshAccount oldActiveAccount )

                Nothing ->
                    ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


modalView : Model -> Html Msg
modalView model =
    case model.modalOpen of
        "account" ->
            case model.activeAccount of
                Just a ->
                    editView a

                Nothing ->
                    div [] []

        "adjustment" ->
            case model.activeAdjustment of
                Just a ->
                    adjustmentEditView a

                Nothing ->
                    div [] []

        _ ->
            div [] []


view : Model -> Html Msg
view model =
    div []
        [ div [] [ renderAccounts model.accounts ]
        , div [] [ modalView model ]
        , p [] [ text (errorMessage model.error) ]
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


deleteAccountAndRefreshAccount : Account -> Cmd Msg
deleteAccountAndRefreshAccount model =
    Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (deleteAccountTask model))


fetchAccounts : Cmd Msg
fetchAccounts =
    Http.send AccountsFetched (get accountsUrl accountsDecoder)


saveAccountAndRefreshAccounts : Account -> Cmd Msg
saveAccountAndRefreshAccounts model =
    case model.id of
        0 ->
            Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveNewAccountTask model))

        _ ->
            Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveAccountTask model))


saveNewAccountTask : Account -> Task Http.Error Account
saveNewAccountTask model =
    toTask (post accountsUrl (jsonBody (Accounts.Utils.encode model)) accountUpdatedDecoder)


refreshAccountsTask : a -> Task Http.Error (List Account)
refreshAccountsTask _ =
    toTask (get accountsUrl accountsDecoder)


saveAccountTask : Account -> Task Http.Error Account
saveAccountTask model =
    toTask (put (accountUrl model.id) (jsonBody (Accounts.Utils.encode model)) accountUpdatedDecoder)


deleteAccountTask : Account -> Task Http.Error String
deleteAccountTask model =
    toTask (delete (accountUrl model.id))


accountsUrl : String
accountsUrl =
    Url.relative
        [ "api", "accounts" ]
        []


accountUrl : Int -> String
accountUrl id =
    Url.relative
        [ "api", "accounts", String.fromInt id ]
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


deleteAdjustmentAndRefreshAdjustments : Adjustment -> Cmd Msg
deleteAdjustmentAndRefreshAdjustments model =
    Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (deleteAdjustmentTask model))


saveAdjustmentAndRefreshAdjustments : Adjustment -> Cmd Msg
saveAdjustmentAndRefreshAdjustments model =
    case model.id of
        0 ->
            Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveNewAdjustmentTask model))

        _ ->
            Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveAdjustmentTask model))


saveAdjustmentTask : Adjustment -> Task Http.Error Adjustment
saveAdjustmentTask model =
    let
        url =
            adjustmentUrl model.accountId model.id

        body =
            jsonBody (Adjustments.Utils.encode model)
    in
    toTask (put url body adjustmentUpdatedDecoder)


deleteAdjustmentTask : Adjustment -> Task Http.Error String
deleteAdjustmentTask model =
    toTask (delete (adjustmentUrl model.accountId model.id))


saveNewAdjustmentTask : Adjustment -> Task Http.Error Adjustment
saveNewAdjustmentTask model =
    toTask (post (adjustmentsUrl model.accountId) (jsonBody (Adjustments.Utils.encode model)) adjustmentUpdatedDecoder)


adjustmentsUrl : Int -> String
adjustmentsUrl accountId =
    Url.relative
        [ "api", "accounts", String.fromInt accountId, "adjustments" ]
        []


adjustmentUrl : Int -> Int -> String
adjustmentUrl accountId id =
    Url.relative
        [ "api", "accounts", String.fromInt accountId, "adjustments", String.fromInt id ]
        []
