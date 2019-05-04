module Main exposing (main)

import Accounts.Messages exposing (..)
import Accounts.Models exposing (..)
import Accounts.Utils exposing (..)
import Accounts.Views exposing (..)
import Browser
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
                    ( model, saveOrCreateAdjustment oldActiveAdjustment )

                Nothing ->
                    ( model, Cmd.none )

        DeleteAdjustment adjustment ->
            ( { model | activeAdjustment = Nothing }, deleteAdjustment adjustment )

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
                    ( model, saveOrCreateAccount oldActiveAccount )

                Nothing ->
                    ( model, Cmd.none )

        DeleteAccount account ->
            ( { model | activeAccount = Nothing }, deleteAccount account )

        DeleteCompleted _ ->
            ( model, refreshAccounts )

        ToggleAdjustmentsFor account ->
            let
                newAccount =
                    { account | adjustmentsVisible = not account.adjustmentsVisible }
            in
            ( { model | accounts = updateNestedAccount newAccount model.accounts }, Cmd.none )

        AccountSaved result ->
            ( model, fetchAccounts )

        AdjustmentSaved result ->
            ( model, fetchAccounts )


updateNestedAccount : Account -> List Account -> List Account
updateNestedAccount account accounts =
    List.map (updateById account) accounts


updateById : Account -> Account -> Account
updateById newAccount account =
    if newAccount.id == account.id then
        newAccount

    else
        account


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

        Just (Http.BadBody message) ->
            message

        _ ->
            "Unknown"


deleteAccount : Account -> Cmd Msg
deleteAccount model =
    delete (accountUrl model.id)


fetchAccounts : Cmd Msg
fetchAccounts =
    get accountsUrl AccountsFetched accountsDecoder


saveOrCreateAccount : Account -> Cmd Msg
saveOrCreateAccount model =
    case model.id of
        0 ->
            saveNewAccount model

        _ ->
            saveAccount model


saveNewAccount : Account -> Cmd Msg
saveNewAccount model =
    let
        url =
            accountsUrl

        body =
            jsonBody (Accounts.Utils.encode model)

        msg =
            AccountSaved

        decoder =
            accountUpdatedDecoder
    in
    post url body msg decoder


refreshAccounts : Cmd Msg
refreshAccounts =
    let
        url =
            accountsUrl

        msg =
            AccountsFetched

        decoder =
            accountsDecoder
    in
    get url msg decoder


saveAccount : Account -> Cmd Msg
saveAccount model =
    let
        url =
            accountUrl model.id

        body =
            jsonBody (Accounts.Utils.encode model)

        msg =
            AccountSaved

        decoder =
            accountUpdatedDecoder
    in
    put url body msg decoder


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


deleteAdjustment : Adjustment -> Cmd Msg
deleteAdjustment model =
    delete (adjustmentUrl model.accountId model.id)


saveOrCreateAdjustment : Adjustment -> Cmd Msg
saveOrCreateAdjustment model =
    case model.id of
        0 ->
            saveAdjustment model

        _ ->
            createAdjustment model


saveAdjustment : Adjustment -> Cmd Msg
saveAdjustment model =
    let
        url =
            adjustmentsUrl model.accountId

        body =
            jsonBody (adjustmentEncode model)
    in
    post url body AdjustmentSaved adjustmentUpdatedDecoder


createAdjustment : Adjustment -> Cmd Msg
createAdjustment model =
    let
        url =
            adjustmentUrl model.accountId model.id

        body =
            jsonBody (adjustmentEncode model)
    in
    put url body AdjustmentSaved adjustmentUpdatedDecoder


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
