module Accounts.Update exposing (accountUrl, accountsUrl, fetchAccounts, put, refreshAccountsTask, saveAccountAndRefreshAccounts, saveAccountTask, update)

import Accounts.Messages
import Accounts.Models exposing (Account)
import Accounts.Utils exposing (accountDecoder, accountUpdatedDecoder, accountsDecoder, adjustmentDecoder, encode)
import Accounts.Views
import Http exposing (get, jsonBody, post, toTask)
import Json.Decode
import Model exposing (Model, Msg(..))
import Task exposing (Task)
import Url.Builder as Url


update : Accounts.Messages.Msg -> Model -> ( Model, Cmd Model.Msg )
update msg model =
    case model.activeAccount of
        Just oldActiveAccount ->
            case msg of
                Accounts.Messages.ToggleDebt ->
                    let
                        newActiveAccount =
                            { oldActiveAccount | debt = not oldActiveAccount.debt }
                    in
                    ( { model | activeAccount = Just newActiveAccount }, Cmd.none )

                Accounts.Messages.NameUpdated newName ->
                    let
                        newActiveAccount =
                            { oldActiveAccount | name = newName }
                    in
                    ( { model | activeAccount = Just newActiveAccount }, Cmd.none )

                Accounts.Messages.BalanceUpdated newBalance ->
                    let
                        newActiveAccount =
                            case String.toFloat newBalance of
                                Just val ->
                                    { oldActiveAccount | balance = val }

                                Nothing ->
                                    { oldActiveAccount | balance = 0 }
                    in
                    ( { model | activeAccount = Just newActiveAccount }, Cmd.none )

                Accounts.Messages.SaveAccount ->
                    ( model, saveAccountAndRefreshAccounts oldActiveAccount )

                Accounts.Messages.DeleteAccount ->
                    ( model, deleteAccountAndRefreshAccount oldActiveAccount )

        Nothing ->
            ( model, Cmd.none )


deleteAccountAndRefreshAccount : Accounts.Models.Account -> Cmd Msg
deleteAccountAndRefreshAccount model =
    Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (deleteAccountTask model))


fetchAccounts : Cmd Msg
fetchAccounts =
    Http.send AccountsFetched (get accountsUrl accountsDecoder)


saveAccountAndRefreshAccounts : Accounts.Models.Account -> Cmd Msg
saveAccountAndRefreshAccounts model =
    case model.id of
        0 ->
            Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveNewAccountTask model))

        _ ->
            Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveAccountTask model))


saveNewAccountTask : Accounts.Models.Account -> Task Http.Error Account
saveNewAccountTask model =
    toTask (post accountsUrl (jsonBody (encode model)) accountUpdatedDecoder)


refreshAccountsTask : a -> Task Http.Error (List Account)
refreshAccountsTask _ =
    toTask (get accountsUrl accountsDecoder)


saveAccountTask : Accounts.Models.Account -> Task Http.Error Account
saveAccountTask model =
    toTask (put (accountUrl model.id) (jsonBody (encode model)) accountUpdatedDecoder)


deleteAccountTask : Accounts.Models.Account -> Task Http.Error String
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


put : String -> Http.Body -> Json.Decode.Decoder Account -> Http.Request Account
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
