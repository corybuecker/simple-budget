module Savings.Views exposing (editView, renderSaving, renderSavings)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))
import Savings.Messages exposing (..)
import Savings.Models exposing (Saving)


editView : Saving -> Html Savings.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.title, onInput Savings.Messages.TitleUpdated ] []
        , input [ type_ "text", value (String.fromFloat model.amount), onInput Savings.Messages.AmountUpdated ] []
        , button [ onClick Savings.Messages.SaveSaving ] [ text "Save" ]
        , button [ onClick Savings.Messages.DeleteSaving ] [ text "Delete" ]
        ]


renderSavings : List Saving -> Html Model.Msg
renderSavings savings =
    div []
        [ button [ class "btn btn-primary", onClick CreateSaving ] [ text "New Saving" ]
        , div []
            [ div [] [ text "Title" ]
            , div [] [ text "Amount" ]
            ]
        , div [] (List.map renderSaving savings)
        ]


renderSaving : Saving -> Html Model.Msg
renderSaving saving =
    div []
        [ div [ onClick (OpenSavingEditor saving) ] [ text saving.title ]
        , div [] [ text (String.fromFloat saving.amount) ]
        ]
